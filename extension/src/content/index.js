/**
 * @file content/index.js
 * @description Content script for scraping LinkedIn profiles.
 */

// Check if we are in a Node.js environment (for testing)
const isNode = typeof module !== 'undefined' && module.exports;

// Mock Logger if not available (e.g. if not bundled)
const Logger = {
    debug: (...args) => console.log('[SuperpoweredCV DEBUG]:', ...args),
    error: (...args) => console.error('[SuperpoweredCV ERROR]:', ...args),
    info: (...args) => console.info('[SuperpoweredCV INFO]:', ...args)
};

if (!isNode) {
    /**
     * Listen for messages from the popup.
     */
    chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
        if (request.action === 'scrape') {
            Logger.info('Received scrape request');
            scrapeProfile()
                .then(data => sendResponse({ data: data }))
                .catch(error => {
                    Logger.error('Scraping failed', error);
                    sendResponse({ error: error.message });
                });
        }
        return true; // Keep channel open for async response
    });
}

/**
 * Scrapes the LinkedIn profile data from the current page.
 * @returns {Promise<Object>} The scraped profile object.
 */
async function scrapeProfile() {
    Logger.debug('Starting profile scrape');
    const url = window.location.href;

    // Check if we are on a details page
    if (url.includes('/details/skills/')) {
        return { skills: await getSkillsFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/languages/')) {
        return { languages: getLanguagesFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/projects/')) {
        return { projects: getProjectsFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/courses/')) {
        return { courses: getCoursesFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/experience/')) {
        return { experience: getExperienceFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/education/')) {
        return { education: getEducationFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/volunteering/')) {
        return { volunteering: getVolunteeringFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/publications/')) {
        return { publications: getPublicationsFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/patents/')) {
        return { patents: getPatentsFromDoc(document), url, isPartial: true };
    }
    if (url.includes('/details/organizations/')) {
        return { organizations: getOrganizationsFromDoc(document), url, isPartial: true };
    }

    // Main Profile Scrape
    const profile = {
        name: getText('h1'),
        headline: getText('.text-body-medium.break-words'),
        location: getText('.text-body-small.inline.t-black--light.break-words'),
        about: getAbout(),
        experience: await getExperience(),
        education: await getEducation(),
        languages: await getLanguages(),
        volunteering: await getVolunteering(), // Replaces 'causes'
        skills: await getSkills(),
        projects: await getProjects(),
        courses: await getCourses(),
        publications: await getPublications(),
        patents: await getPatents(),
        organizations: await getOrganizations(),
        contactInfo: await getContactInfo(),
        url: url
    };
    Logger.debug('Profile scraped:', profile);
    return profile;
}

/**
 * Helper to get text content from a selector.
 * @param {string} selector - CSS selector.
 * @returns {string} The trimmed text content or empty string.
 */
function getText(selector) {
    if (typeof document === 'undefined') return '';
    const el = document.querySelector(selector);
    return el ? el.innerText.trim() : '';
}

/**
 * Helper to fetch a URL and return the document.
 * @param {string} url 
 * @returns {Promise<Document>}
 */
async function fetchDocument(url) {
    try {
        const response = await fetch(url);
        const text = await response.text();
        const parser = new DOMParser();
        return parser.parseFromString(text, 'text/html');
    } catch (e) {
        Logger.error(`Failed to fetch ${url}`, e);
        return null;
    }
}

/**
 * Scrapes the 'About' section.
 * @returns {string} The about text.
 */
function getAbout() {
    if (typeof document === 'undefined') return '';
    const section = document.getElementById('about');
    if (section) {
        const parent = section.closest('.artdeco-card');
        if (parent) {
            const textContainer = parent.querySelector('.inline-show-more-text--is-collapsed, .inline-show-more-text--is-expanded');
            if (textContainer) return textContainer.innerText.trim();
            
            const text = parent.querySelector('.inline-show-more-text span[aria-hidden="true"]');
            if (text) return text.innerText.trim();
        }
    }
    return '';
}

/**
 * Scrapes the 'Experience' section.
 * @returns {Promise<Array<Object>>} List of experience items.
 */
async function getExperience() {
    if (typeof document === 'undefined') return [];
    
    // Check for "Show all experiences" link
    const footerLink = document.querySelector('#experience')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getExperienceFromDoc(doc);
    }

    return getExperienceFromDoc(document);
}

/**
 * Helper to extract experience items from a document.
 * @param {Document} doc - The document to scrape (main page or details page).
 * @returns {Array<Object>} List of experience items.
 */
function getExperienceFromDoc(doc) {
    const experiences = [];
    // If on details page, the structure is slightly different (pvs-list)
    // But usually the list items share similar structure or we can target .pvs-list__paged-list-item
    
    // Try generic PVS list approach first (works for both main and details usually)
    const items = doc.querySelectorAll('.pvs-list__paged-list-item, li.artdeco-list__item');
    
    // Filter to ensure we are looking at experience items if on main page
    // On main page, we need to scope to #experience section
    let targetItems = items;
    if (doc === document) { // Main page
        const section = doc.getElementById('experience');
        if (!section) return [];
        const parent = section.closest('.artdeco-card');
        if (!parent) return [];
        targetItems = parent.querySelectorAll('li.artdeco-list__item');
    }

    targetItems.forEach(item => {
        // Selectors for Title, Company, etc.
        // These selectors are fragile and change often. 
        // We look for the first strong text as title, second as company, etc.
        
        const spans = item.querySelectorAll('span[aria-hidden="true"]');
        if (spans.length < 1) return;

        // Heuristic: 
        // 1. Title is usually the first bold/strong element or the first span in the first link.
        // 2. Company is usually the second line.
        // 3. Date/Tenure is usually the third line (t-black--light).
        
        // Let's try to find specific classes if possible, but fall back to order.
        const titleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]') || spans[0];
        const companyEl = item.querySelector('span.t-14.t-normal span[aria-hidden="true"]'); // This might be missing if multiple roles at same company
        
        // If multiple roles at same company, the structure is nested.
        // For now, let's stick to the flat structure or the main role.
        
        // Refined extraction logic
        let title = titleEl ? titleEl.innerText.trim() : '';
        let company = companyEl ? companyEl.innerText.trim() : '';
        
        // Meta data (Date, Location)
        const metaEls = item.querySelectorAll('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        let dateRange = '';
        let tenure = '';
        let location = '';

        if (metaEls.length > 0) {
            const dateAndTenure = metaEls[0].innerText.trim();
            const parts = dateAndTenure.split('·').map(s => s.trim());
            dateRange = parts[0] || '';
            if (parts.length > 1) {
                tenure = parts[1];
                if (dateRange.toLowerCase().includes('present')) {
                    tenure = 'over ' + tenure;
                }
            }
        }
        if (metaEls.length > 1) {
            location = metaEls[1].innerText.trim();
        }

        // Description
        let description = '';
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const cleanSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = cleanSpan ? cleanSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        // Skills
        let skills = [];
        const skillsContainer = Array.from(item.querySelectorAll('div.display-flex')).find(div => div.innerText.includes('Skills:'));
        if (skillsContainer) {
            const fullText = skillsContainer.innerText.trim();
            const skillsText = fullText.replace(/^Skills:\s*/i, '');
            skills = skillsText.split('·').map(s => s.trim());
        }

        if (title) {
            experiences.push({ title, company, date_range: dateRange, tenure, location, description, skills });
        }
    });

    return experiences;
}

/**
 * Helper to extract volunteering items from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of volunteering items.
 */
function getVolunteeringFromDoc(doc) {
    const items = getSectionItems(doc, 'volunteering');
    return items.map(item => {
        const roleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const orgEl = item.querySelector('span.t-14.t-normal span[aria-hidden="true"]');
        const metaEls = item.querySelectorAll('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        const descriptionEl = item.querySelector('.inline-show-more-text span[aria-hidden="true"]');

        let dateRange = '';
        let tenure = '';
        if (metaEls.length > 0) {
            const dateAndTenure = metaEls[0].innerText.trim();
            const parts = dateAndTenure.split('·').map(s => s.trim());
            dateRange = parts[0] || '';
            if (parts.length > 1) tenure = parts[1];
        }

        return {
            role: roleEl ? roleEl.innerText.trim() : '',
            organization: orgEl ? orgEl.innerText.trim() : '',
            date_range: dateRange,
            tenure: tenure,
            description: descriptionEl ? descriptionEl.innerText.trim() : ''
        };
    }).filter(i => i.role);
}

/**
 * Scrapes the 'Languages' section.
 * @returns {Promise<Array<Object>>} List of languages.
 */
async function getLanguages() {
    if (typeof document === 'undefined') return [];

    const footerLink = document.querySelector('#languages')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getLanguagesFromDoc(doc);
    }
    return getLanguagesFromDoc(document);
}

/**
 * Helper to extract languages from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of languages.
 */
function getLanguagesFromDoc(doc) {
    const items = getSectionItems(doc, 'languages');
    return items.map(item => {
        const nameEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const proficiencyEl = item.querySelector('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        return {
            name: nameEl ? nameEl.innerText.trim() : '',
            proficiency: proficiencyEl ? proficiencyEl.innerText.trim() : ''
        };
    }).filter(i => i.name);
}

/**
 * Scrapes the 'Education' section.
 * @returns {Promise<Array<Object>>} List of education items.
 */
async function getEducation() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#education')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getEducationFromDoc(doc);
    }
    return getEducationFromDoc(document);
}

/**
 * Helper to extract education items from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of education items.
 */
function getEducationFromDoc(doc) {
    const items = getSectionItems(doc, 'education');
    return items.map(item => {
        const schoolEl = item.querySelector('.display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]');
        const degreeEl = item.querySelector('span.t-14.t-normal span[aria-hidden="true"]');
        return {
            school: schoolEl ? schoolEl.innerText.trim() : '',
            degree: degreeEl ? degreeEl.innerText.trim() : ''
        };
    }).filter(i => i.school);
}

/**
 * Scrapes the 'Skills' section.
 * @returns {Promise<Array<string>>} List of skills.
 */
async function getSkills() {
    if (typeof document === 'undefined') return [];

    const footerLink = document.querySelector('#skills')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getSkillsFromDoc(doc);
    }
    return getSkillsFromDoc(document);
}

/**
 * Helper to extract skills from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<string>} List of skills.
 */
function getSkillsFromDoc(doc) {
    const items = getSectionItems(doc, 'skills');
    return items.map(item => {
        const skillEl = item.querySelector('.display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]') || item.querySelector('span[aria-hidden="true"]');
        return skillEl ? skillEl.innerText.trim() : '';
    }).filter(s => s);
}

/**
 * Scrapes the 'Projects' section.
 * @returns {Promise<Array<Object>>} List of projects.
 */
async function getProjects() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#projects')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getProjectsFromDoc(doc);
    }
    return getProjectsFromDoc(document);
}

function getProjectsFromDoc(doc) {
    const items = getSectionItems(doc, 'projects');
    return items.map(item => {
        const titleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const dateEl = item.querySelector('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        const descriptionEl = item.querySelector('.inline-show-more-text span[aria-hidden="true"]');
        const linkEl = item.querySelector('a.optional-action-target-wrapper');

        return {
            title: titleEl ? titleEl.innerText.trim() : '',
            date: dateEl ? dateEl.innerText.trim() : '',
            description: descriptionEl ? descriptionEl.innerText.trim() : '',
            link: linkEl ? linkEl.href : ''
        };
    }).filter(i => i.title);
}

/**
 * Scrapes the 'Courses' section.
 * @returns {Promise<Array<Object>>} List of courses.
 */
async function getCourses() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#courses')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getCoursesFromDoc(doc);
    }
    return getCoursesFromDoc(document);
}

function getCoursesFromDoc(doc) {
    const items = getSectionItems(doc, 'courses');
    return items.map(item => {
        const nameEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const numberEl = item.querySelector('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        return {
            name: nameEl ? nameEl.innerText.trim() : '',
            number: numberEl ? numberEl.innerText.trim() : ''
        };
    }).filter(i => i.name);
}

/**
 * Scrapes the 'Publications' section.
 * @returns {Promise<Array<Object>>} List of publications.
 */
async function getPublications() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#publications')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getPublicationsFromDoc(doc);
    }
    return getPublicationsFromDoc(document);
}

function getPublicationsFromDoc(doc) {
    const items = getSectionItems(doc, 'publications');
    return items.map(item => {
        const titleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const dateEl = item.querySelector('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        const descriptionEl = item.querySelector('.inline-show-more-text span[aria-hidden="true"]');
        const linkEl = item.querySelector('a.optional-action-target-wrapper');

        return {
            title: titleEl ? titleEl.innerText.trim() : '',
            date: dateEl ? dateEl.innerText.trim() : '',
            description: descriptionEl ? descriptionEl.innerText.trim() : '',
            link: linkEl ? linkEl.href : ''
        };
    }).filter(i => i.title);
}

/**
 * Scrapes the 'Patents' section.
 * @returns {Promise<Array<Object>>} List of patents.
 */
async function getPatents() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#patents')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getPatentsFromDoc(doc);
    }
    return getPatentsFromDoc(document);
}

function getPatentsFromDoc(doc) {
    const items = getSectionItems(doc, 'patents');
    return items.map(item => {
        const titleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const numberEl = item.querySelector('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        const descriptionEl = item.querySelector('.inline-show-more-text span[aria-hidden="true"]');
        
        return {
            title: titleEl ? titleEl.innerText.trim() : '',
            number: numberEl ? numberEl.innerText.trim() : '',
            description: descriptionEl ? descriptionEl.innerText.trim() : ''
        };
    }).filter(i => i.title);
}

/**
 * Scrapes the 'Organizations' section.
 * @returns {Promise<Array<Object>>} List of organizations.
 */
async function getOrganizations() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#organizations')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getOrganizationsFromDoc(doc);
    }
    return getOrganizationsFromDoc(document);
}

function getOrganizationsFromDoc(doc) {
    const items = getSectionItems(doc, 'organizations');
    return items.map(item => {
        const nameEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const roleEl = item.querySelector('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        const dateEl = item.querySelectorAll('span.t-14.t-normal.t-black--light span[aria-hidden="true"]')[1];
        const descriptionEl = item.querySelector('.inline-show-more-text span[aria-hidden="true"]');

        return {
            name: nameEl ? nameEl.innerText.trim() : '',
            role: roleEl ? roleEl.innerText.trim() : '',
            date: dateEl ? dateEl.innerText.trim() : '',
            description: descriptionEl ? descriptionEl.innerText.trim() : ''
        };
    }).filter(i => i.name);
}

/**
 * Scrapes Contact Info from the overlay.
 * @returns {Promise<Object>} Contact info object.
 */
async function getContactInfo() {
    if (typeof window === 'undefined') return {};
    
    // Construct overlay URL
    // URL format: https://www.linkedin.com/in/{slug}/overlay/contact-info/
    // We need to extract the slug from the current URL
    const match = window.location.href.match(/\/in\/([^\/]+)/);
    if (!match) return {};
    
    const slug = match[1];
    const url = `https://www.linkedin.com/in/${slug}/overlay/contact-info/`;
    
    const doc = await fetchDocument(url);
    if (!doc) return {};

    const contactInfo = {};
    
    // Email
    const emailSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Email'));
    if (emailSection) {
        const link = emailSection.querySelector('a');
        if (link) contactInfo.email = link.innerText.trim();
    }

    // Phone
    const phoneSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Phone'));
    if (phoneSection) {
        const items = phoneSection.querySelectorAll('li');
        contactInfo.phone = Array.from(items).map(i => i.innerText.trim());
    }

    // Website
    const websiteSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Website'));
    if (websiteSection) {
        const items = websiteSection.querySelectorAll('li a');
        contactInfo.websites = Array.from(items).map(i => i.href);
    }

    // Twitter/X
    const twitterSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Twitter'));
    if (twitterSection) {
        const link = twitterSection.querySelector('a');
        if (link) contactInfo.twitter = link.href;
    }

    return contactInfo;
}

/**
 * Helper to get items from a section, handling both main page and details page structures.
 * @param {Document} doc 
 * @param {string} sectionId 
 * @returns {Array<Element>}
 */
function getSectionItems(doc, sectionId) {
    let items = [];
    
    // If we are on the main page, find the section by ID
    if (doc === document) {
        const section = doc.getElementById(sectionId);
        if (section) {
            const parent = section.closest('.artdeco-card');
            if (parent) {
                items = parent.querySelectorAll('li.artdeco-list__item');
            }
        }
    } else {
        // If we are on a details page (fetched doc), the structure is usually a pvs-list
        items = doc.querySelectorAll('.pvs-list__paged-list-item, li.artdeco-list__item');
    }
    
    return Array.from(items);
}
            const footer = parent.querySelector('.pvs-list__footer-wrapper a');
            if (footer && footer.href) {
                try {
                    Logger.debug('Fetching full skills from:', footer.href);
                    const response = await fetch(footer.href);
                    const text = await response.text();
                    const parser = new DOMParser();
                    const doc = parser.parseFromString(text, 'text/html');
                    return getSkillsFromDoc(doc);
                } catch (e) {
                    Logger.error('Failed to fetch full skills', e);
                }
            }

            // Fallback to visible skills
            const items = parent.querySelectorAll('.display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]');
            items.forEach(item => {
                skills.push(item.innerText.trim());
            });
        }
    }
    return skills;
}

/**
 * Helper to extract skills from a document (main or details page).
 * @param {Document} doc - The document to scrape.
 * @returns {Array<string>} List of skills.
 */
function getSkillsFromDoc(doc) {
    const skills = [];
    // On details page, skills are usually in a list
    // Selector for skills in the details list
    const items = doc.querySelectorAll('.pvs-list__paged-list-item .display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]');
    
    // If that selector fails, try a more generic one often found in details pages
    if (items.length === 0) {
         const genericItems = doc.querySelectorAll('.pvs-list__paged-list-item span.visually-hidden'); // Sometimes hidden
         // Or the visible text
         const visibleItems = doc.querySelectorAll('.pvs-list__paged-list-item div.display-flex.align-items-center > span[aria-hidden="true"]');
         
         visibleItems.forEach(item => {
             const text = item.innerText.trim();
             if (text && !skills.includes(text)) skills.push(text);
         });
    } else {
        items.forEach(item => {
            const text = item.innerText.trim();
            if (text && !skills.includes(text)) skills.push(text);
        });
    }
    
    return skills;
}

// Export for testing
if (isNode) {
    module.exports = {
        scrapeProfile,
        getText,
        getAbout,
        getExperience,
        getEducation,
        getSkills
    };
}
