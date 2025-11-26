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
        experience: await scrapeSection('Experience', getExperience),
        education: await scrapeSection('Education', getEducation),
        languages: await scrapeSection('Languages', getLanguages),
        volunteering: await scrapeSection('Volunteering', getVolunteering),
        skills: await scrapeSection('Skills', getSkills),
        projects: await scrapeSection('Projects', getProjects),
        courses: await scrapeSection('Courses', getCourses),
        publications: await scrapeSection('Publications', getPublications),
        patents: await scrapeSection('Patents', getPatents),
        organizations: await scrapeSection('Organizations', getOrganizations),
        contactInfo: await scrapeSection('Contact Info', getContactInfo),
        url: url
    };
    Logger.debug('Profile scraped:', profile);
    return profile;
}

/**
 * Helper to report progress and execute a scraper function.
 * @param {string} name - Name of the section being scraped.
 * @param {Function} scraperFn - The scraper function to call.
 * @returns {Promise<any>} The result of the scraper function.
 */
async function scrapeSection(name, scraperFn) {
    if (!isNode) {
        chrome.runtime.sendMessage({ action: 'progress', message: `Scraping ${name}...` });
    }
    try {
        return await scraperFn();
    } catch (e) {
        Logger.error(`Error scraping ${name}`, e);
        return []; // Return empty array/object on failure to prevent total crash
    }
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
            // Try to find the visible text span specifically
            const visibleSpan = parent.querySelector('.inline-show-more-text span[aria-hidden="true"]');
            if (visibleSpan) return visibleSpan.innerText.trim();

            // Fallback to container but try to exclude visually-hidden
            const textContainer = parent.querySelector('.inline-show-more-text--is-collapsed, .inline-show-more-text--is-expanded');
            if (textContainer) {
                // Clone to avoid modifying DOM
                const clone = textContainer.cloneNode(true);
                const hidden = clone.querySelectorAll('.visually-hidden');
                hidden.forEach(el => el.remove());
                return clone.innerText.trim();
            }
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
    const items = getSectionItems(doc, 'experience');
    const experiences = [];

    items.forEach(item => {
        // Generic extraction strategy:
        // 1. Get all visible spans (aria-hidden="true")
        // 2. Filter out empty/separator spans
        // 3. Map by position (Title, Company, Date, Location)
        
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        if (texts.length < 2) return; // Need at least Title and Company

        let title = texts[0];
        let company = texts[1];
        let dateRange = '';
        let tenure = '';
        let location = '';
        let description = '';

        // Try to identify date/tenure by pattern
        // Date usually contains year (19xx or 20xx) or month names
        // Tenure usually contains "yr" or "mos"
        
        // Find index of date-like string
        const dateIndex = texts.findIndex(t => /\d{4}|Present/i.test(t) && (t.includes(' - ') || t.includes('·')));
        
        if (dateIndex > 1) {
            // If we found a date later, everything before it might be title/company parts
            // But usually Title is 0, Company is 1.
            const dateText = texts[dateIndex];
            const parts = dateText.split('·').map(s => s.trim());
            dateRange = parts[0] || '';
            if (parts.length > 1) tenure = parts[1];
            
            if (texts[dateIndex + 1]) location = texts[dateIndex + 1];
        } else if (texts[2]) {
             // Fallback: assume 3rd item is date
             const parts = texts[2].split('·').map(s => s.trim());
             dateRange = parts[0];
             if (parts.length > 1) tenure = parts[1];
             if (texts[3]) location = texts[3];
        }

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        // Skills
        let skills = [];
        const skillsContainer = Array.from(item.querySelectorAll('div')).find(div => div.innerText.includes('Skills:'));
        if (skillsContainer) {
            // Be careful not to grab the whole item text
            // Usually skills are in a distinct div
            const fullText = skillsContainer.innerText.trim();
            // Check if this div is actually small enough to be just skills
            if (fullText.length < 500) {
                 const skillsText = fullText.replace(/^Skills:\s*/i, '');
                 skills = skillsText.split('·').map(s => s.trim());
            }
        }

        experiences.push({ title, company, date_range: dateRange, tenure, location, description, skills });
    });

    return experiences;
}

/**
 * Scrapes the 'Volunteering' section.
 * @returns {Promise<Array<Object>>} List of volunteering items.
 */
async function getVolunteering() {
    if (typeof document === 'undefined') return [];
    
    const footerLink = document.querySelector('#volunteering')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    if (footerLink && footerLink.href) {
        const doc = await fetchDocument(footerLink.href);
        if (doc) return getVolunteeringFromDoc(doc);
    }
    return getVolunteeringFromDoc(document);
}

/**
 * Helper to extract volunteering items from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of volunteering items.
 */
function getVolunteeringFromDoc(doc) {
    const items = getSectionItems(doc, 'volunteering');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let role = texts[0] || '';
        let organization = texts[1] || '';
        let dateRange = '';
        let tenure = '';
        let description = '';

        // Date logic
        const dateIndex = texts.findIndex(t => /\d{4}|Present/i.test(t));
        if (dateIndex > 1) {
            const parts = texts[dateIndex].split('·').map(s => s.trim());
            dateRange = parts[0];
            if (parts.length > 1) tenure = parts[1];
        }

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        return { role, organization, date_range: dateRange, tenure, description };
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
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');
        
        return {
            name: texts[0] || '',
            proficiency: texts[1] || ''
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
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let school = texts[0] || '';
        let degree = texts[1] || '';
        let dateRange = '';
        let grade = '';
        let description = '';

        // Try to find date
        const dateIndex = texts.findIndex(t => /\d{4}/.test(t));
        if (dateIndex > 1) {
            dateRange = texts[dateIndex];
        }

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        return { school, degree, date_range: dateRange, description };
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

/**
 * Helper to extract projects from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of projects.
 */
function getProjectsFromDoc(doc) {
    const items = getSectionItems(doc, 'projects');
    return items.map(item => {
        // Generic extraction
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');
        
        let title = texts[0] || '';
        let date = '';
        let description = '';
        let link = '';

        // Try to find date
        const dateIndex = texts.findIndex(t => /\d{4}|Present/i.test(t));
        if (dateIndex > 0) date = texts[dateIndex];

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        // Link
        const linkEl = item.querySelector('a.optional-action-target-wrapper');
        if (linkEl) link = linkEl.href;

        return { title, date, description, link };
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
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');
        
        return {
            name: texts[0] || '',
            number: texts[1] || ''
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
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let title = texts[0] || '';
        let date = '';
        let description = '';
        let link = '';

        // Date logic
        const dateIndex = texts.findIndex(t => /\d{4}/.test(t));
        if (dateIndex > 0) date = texts[dateIndex];

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        // Link
        const linkEl = item.querySelector('a.optional-action-target-wrapper');
        if (linkEl) link = linkEl.href;

        return { title, date, description, link };
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
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let title = texts[0] || '';
        let number = texts[1] || '';
        let description = '';

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }
        
        return { title, number, description };
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
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let name = texts[0] || '';
        let role = texts[1] || '';
        let date = '';
        let description = '';

        // Date logic
        const dateIndex = texts.findIndex(t => /\d{4}/.test(t));
        if (dateIndex > 1) date = texts[dateIndex];

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        return { name, role, date, description };
    }).filter(i => i.name);
}

/**
 * Scrapes Contact Info from the overlay.
 * @returns {Promise<Object>} Contact info object.
 */
async function getContactInfo() {
    if (typeof window === 'undefined') return {};
    
    // Construct overlay URL
    const match = window.location.href.match(/\/in\/([^\/]+)/);
    if (!match) return {};
    
    const slug = match[1];
    const url = `https://www.linkedin.com/in/${slug}/overlay/contact-info/`;
    
    const doc = await fetchDocument(url);
    if (!doc) return {};

    const contactInfo = {};
    
    // Generic section parser for contact info
    // Look for sections by header text
    const sections = Array.from(doc.querySelectorAll('section'));
    
    sections.forEach(section => {
        const header = section.querySelector('h3');
        if (!header) return;
        const headerText = header.innerText.toLowerCase();
        
        if (headerText.includes('email')) {
            const link = section.querySelector('a');
            if (link) contactInfo.email = link.innerText.trim();
        } else if (headerText.includes('phone')) {
            const items = section.querySelectorAll('li, span.t-14');
            contactInfo.phone = Array.from(items).map(i => i.innerText.trim()).filter(t => t);
        } else if (headerText.includes('website')) {
            const items = section.querySelectorAll('a');
            contactInfo.websites = Array.from(items).map(i => i.href);
        } else if (headerText.includes('twitter') || headerText.includes('x')) { // X (Twitter)
            const link = section.querySelector('a');
            if (link) contactInfo.twitter = link.href;
        }
    });

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
