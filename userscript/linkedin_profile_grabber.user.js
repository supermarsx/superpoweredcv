// ==UserScript==
// @name         SuperpoweredCV Profile Grabber
// @namespace    http://tampermonkey.net/
// @version      1.0
// @description  Scrape LinkedIn profiles for SuperpoweredCV analysis.
// @author       SuperpoweredCV
// @match        https://www.linkedin.com/in/*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=linkedin.com
// @grant        none
// ==/UserScript==

(function() {
    'use strict';

    // --- Styles ---
    const styles = `
        #spcv-grab-btn {
            position: fixed;
            bottom: 20px;
            right: 20px;
            z-index: 9999;
            padding: 15px 20px;
            background-color: #ff3300;
            color: #000;
            border: 4px solid #000;
            font-family: 'Courier New', Courier, monospace;
            font-weight: bold;
            font-size: 16px;
            text-transform: uppercase;
            cursor: pointer;
            box-shadow: 8px 8px 0px rgba(0,0,0,0.8);
            transition: all 0.1s;
        }
        #spcv-grab-btn:hover {
            transform: translate(-2px, -2px);
            box-shadow: 12px 12px 0px rgba(0,0,0,0.8);
            background-color: #000;
            color: #ff3300;
            border-color: #ff3300;
        }
        #spcv-grab-btn:active {
            transform: translate(0, 0);
            box-shadow: 4px 4px 0px rgba(0,0,0,0.8);
        }
    `;

    // --- UI Injection ---
    /**
     * Injects the scraping button and styles into the page.
     */
    function injectUI() {
        const styleSheet = document.createElement("style");
        styleSheet.innerText = styles;
        document.head.appendChild(styleSheet);

        const btn = document.createElement("button");
        btn.id = "spcv-grab-btn";
        btn.innerText = "INITIATE_DUMP";
        btn.onclick = handleGrabClick;
        document.body.appendChild(btn);
    }

    // --- Handlers ---
    /**
     * Handles the click event on the scrape button.
     */
    async function handleGrabClick() {
        const btn = document.getElementById("spcv-grab-btn");
        btn.innerText = "SCRAPING...";
        
        try {
            // Wait a bit for dynamic content if needed, but usually user clicks when ready
            setTimeout(async () => {
                try {
                    const data = await scrapeProfile();
                    downloadJSON(data, `profile_${data.name || 'unknown'}.json`);
                    btn.innerText = "DONE!";
                    setTimeout(() => btn.innerText = "INITIATE_DUMP", 3000);
                } catch (e) {
                    console.error(e);
                    btn.innerText = "ERROR";
                    alert("Scraping failed: " + e.message);
                }
            }, 500);
        } catch (e) {
            console.error(e);
            btn.innerText = "ERROR";
            alert("Scraping failed: " + e.message);
        }
    }

    /**
     * Triggers a download of the scraped data as a JSON file.
     * @param {Object} data - The data to download.
     * @param {string} filename - The filename to save as.
     */
    function downloadJSON(data, filename) {
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    // --- Scraping Logic ---
    /**
     * Main function to scrape the profile.
     * @returns {Promise<Object>} The scraped profile data.
     */
    async function scrapeProfile() {
        console.log('[SuperpoweredCV] Starting scrape...');
        const url = window.location.href;

        // Check if we are on a details page
        if (url.includes('/details/skills/')) return { skills: await getSkillsFromDoc(document), url, isPartial: true };
        if (url.includes('/details/languages/')) return { languages: getLanguagesFromDoc(document), url, isPartial: true };
        if (url.includes('/details/projects/')) return { projects: getProjectsFromDoc(document), url, isPartial: true };
        if (url.includes('/details/courses/')) return { courses: getCoursesFromDoc(document), url, isPartial: true };
        if (url.includes('/details/experience/')) return { experience: getExperienceFromDoc(document), url, isPartial: true };
        if (url.includes('/details/education/')) return { education: getEducationFromDoc(document), url, isPartial: true };
        if (url.includes('/details/volunteering/')) return { volunteering: getVolunteeringFromDoc(document), url, isPartial: true };
        if (url.includes('/details/publications/')) return { publications: getPublicationsFromDoc(document), url, isPartial: true };
        if (url.includes('/details/patents/')) return { patents: getPatentsFromDoc(document), url, isPartial: true };
        if (url.includes('/details/organizations/')) return { organizations: getOrganizationsFromDoc(document), url, isPartial: true };

        const profile = {
            name: getText('h1'),
            headline: getText('.text-body-medium.break-words'),
            location: getText('.text-body-small.inline.t-black--light.break-words'),
            about: getAbout(),
            experience: await getExperience(),
            education: await getEducation(),
            languages: await getLanguages(),
            volunteering: await getVolunteering(),
            skills: await getSkills(),
            projects: await getProjects(),
            courses: await getCourses(),
            publications: await getPublications(),
            patents: await getPatents(),
            organizations: await getOrganizations(),
            contactInfo: await getContactInfo(),
            url: url
        };
        console.log('[SuperpoweredCV] Scraped:', profile);
        return profile;
    }

    /**
     * Helper to safely get text content from an element.
     * @param {string} selector - The selector to find the element.
     * @returns {string} The text content or empty string.
     */
    function getText(selector) {
        const el = document.querySelector(selector);
        return el ? el.innerText.trim() : '';
    }

    /**
     * Fetches a document from a URL.
     * @param {string} url - The URL to fetch.
     * @returns {Promise<Document|null>} The fetched document or null on error.
     */
    async function fetchDocument(url) {
        try {
            const response = await fetch(url);
            const text = await response.text();
            const parser = new DOMParser();
            return parser.parseFromString(text, 'text/html');
        } catch (e) {
            console.error(`Failed to fetch ${url}`, e);
            return null;
        }
    }

    /**
     * Scrapes the 'About' section.
     * @returns {string} The about text.
     */
    function getAbout() {
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
        const footerLink = document.querySelector('#experience')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
        if (footerLink && footerLink.href) {
            const doc = await fetchDocument(footerLink.href);
            if (doc) return getExperienceFromDoc(doc);
        }
        return getExperienceFromDoc(document);
    }

    /**
     * Helper to extract experience items from a document.
     * @param {Document} doc - The document to scrape.
     * @returns {Array<Object>} List of experience items.
     */
    function getExperienceFromDoc(doc) {
        const experiences = [];
        let targetItems = doc.querySelectorAll('.pvs-list__paged-list-item, li.artdeco-list__item');
        if (doc === document) {
            const section = doc.getElementById('experience');
            if (!section) return [];
            const parent = section.closest('.artdeco-card');
            if (!parent) return [];
            targetItems = parent.querySelectorAll('li.artdeco-list__item');
        }

        targetItems.forEach(item => {
            const spans = item.querySelectorAll('span[aria-hidden="true"]');
            if (spans.length < 1) return;

            const titleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]') || spans[0];
            const companyEl = item.querySelector('span.t-14.t-normal span[aria-hidden="true"]');
            
            let title = titleEl ? titleEl.innerText.trim() : '';
            let company = companyEl ? companyEl.innerText.trim() : '';
            
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
            if (metaEls.length > 1) location = metaEls[1].innerText.trim();

            let description = '';
            const descriptionEl = item.querySelector('.inline-show-more-text');
            if (descriptionEl) {
                const cleanSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
                description = cleanSpan ? cleanSpan.innerText.trim() : descriptionEl.innerText.trim();
            }

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
     * Scrapes the 'Volunteering' section.
     * @returns {Promise<Array<Object>>} List of volunteering items.
     */
    async function getVolunteering() {
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
        const footerLink = document.querySelector('#courses')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
        if (footerLink && footerLink.href) {
            const doc = await fetchDocument(footerLink.href);
            if (doc) return getCoursesFromDoc(doc);
        }
        return getCoursesFromDoc(document);
    }

    /**
     * Helper to extract courses from a document.
     * @param {Document} doc - The document to scrape.
     * @returns {Array<Object>} List of courses.
     */
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
        const footerLink = document.querySelector('#publications')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
        if (footerLink && footerLink.href) {
            const doc = await fetchDocument(footerLink.href);
            if (doc) return getPublicationsFromDoc(doc);
        }
        return getPublicationsFromDoc(document);
    }

    /**
     * Helper to extract publications from a document.
     * @param {Document} doc - The document to scrape.
     * @returns {Array<Object>} List of publications.
     */
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
        const footerLink = document.querySelector('#patents')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
        if (footerLink && footerLink.href) {
            const doc = await fetchDocument(footerLink.href);
            if (doc) return getPatentsFromDoc(doc);
        }
        return getPatentsFromDoc(document);
    }

    /**
     * Helper to extract patents from a document.
     * @param {Document} doc - The document to scrape.
     * @returns {Array<Object>} List of patents.
     */
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
        const footerLink = document.querySelector('#organizations')?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
        if (footerLink && footerLink.href) {
            const doc = await fetchDocument(footerLink.href);
            if (doc) return getOrganizationsFromDoc(doc);
        }
        return getOrganizationsFromDoc(document);
    }

    /**
     * Helper to extract organizations from a document.
     * @param {Document} doc - The document to scrape.
     * @returns {Array<Object>} List of organizations.
     */
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
     * Scrapes contact info from the overlay.
     * @returns {Promise<Object>} Contact info object.
     */
    async function getContactInfo() {
        const match = window.location.href.match(/\/in\/([^\/]+)/);
        if (!match) return {};
        
        const slug = match[1];
        const url = `https://www.linkedin.com/in/${slug}/overlay/contact-info/`;
        
        const doc = await fetchDocument(url);
        if (!doc) return {};

        const contactInfo = {};
        
        const emailSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Email'));
        if (emailSection) {
            const link = emailSection.querySelector('a');
            if (link) contactInfo.email = link.innerText.trim();
        }

        const phoneSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Phone'));
        if (phoneSection) {
            const items = phoneSection.querySelectorAll('li');
            contactInfo.phone = Array.from(items).map(i => i.innerText.trim());
        }

        const websiteSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Website'));
        if (websiteSection) {
            const items = websiteSection.querySelectorAll('li a');
            contactInfo.websites = Array.from(items).map(i => i.href);
        }

        const twitterSection = Array.from(doc.querySelectorAll('section')).find(s => s.innerText.includes('Twitter'));
        if (twitterSection) {
            const link = twitterSection.querySelector('a');
            if (link) contactInfo.twitter = link.href;
        }

        return contactInfo;
    }

    /**
     * Helper to get list items for a section.
     * @param {Document} doc - The document to search.
     * @param {string} sectionId - The ID of the section (e.g., 'experience').
     * @returns {Array<Element>} List of list item elements.
     */
    function getSectionItems(doc, sectionId) {
        let items = [];
        if (doc === document) {
            const section = doc.getElementById(sectionId);
            if (section) {
                const parent = section.closest('.artdeco-card');
                if (parent) {
                    items = parent.querySelectorAll('li.artdeco-list__item');
                }
            }
        } else {
            items = doc.querySelectorAll('.pvs-list__paged-list-item, li.artdeco-list__item');
        }
        return Array.from(items);
    }

    // --- Init ---
    // Wait for page load
    window.addEventListener('load', injectUI);
    // Also try injecting immediately if document is ready (for SPA navigation sometimes)
    if (document.readyState === 'complete' || document.readyState === 'interactive') {
        injectUI();
    }

})();
