const content = require('../src/content/index.js');

describe('Content Script', () => {
    test('getText returns empty string if document is undefined', () => {
        // We are in jsdom environment, so document exists.
        // We can mock querySelector to return null
        jest.spyOn(document, 'querySelector').mockReturnValue(null);
        expect(content.getText('h1')).toBe('');
    });

    test('getText returns text content', () => {
        const dummyEl = document.createElement('div');
        dummyEl.textContent = '  Hello World  ';
        jest.spyOn(document, 'querySelector').mockReturnValue(dummyEl);
        expect(content.getText('h1')).toBe('Hello World');
    });

    test('getAbout extracts description', () => {
        document.body.innerHTML = `
            <div class="artdeco-card">
                <div id="about"></div>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">About Me Text</span>
                </div>
            </div>
        `;
        expect(content.getAbout()).toBe('About Me Text');
    });

    test('getExperienceFromDoc extracts items', () => {
        document.body.innerHTML = `
            <div id="experience">
                <div class="pvs-list__paged-list-item">
                    <span aria-hidden="true">Software Engineer</span>
                    <span aria-hidden="true">Tech Corp</span>
                    <span aria-hidden="true">2020 - Present · 3 yrs</span>
                    <span aria-hidden="true">New York</span>
                    <div class="inline-show-more-text">
                        <span aria-hidden="true">Built cool stuff.</span>
                    </div>
                </div>
            </div>
        `;
        // We need to mock getSectionItems or ensure the DOM structure matches what getSectionItems expects.
        // getSectionItems looks for #experience -> closest card -> li.artdeco-list__item OR .pvs-list__paged-list-item
        // Since we are passing 'doc' as document, it uses the first branch (main page).
        // But getExperienceFromDoc calls getSectionItems(doc, 'experience').
        
        // Let's mock getSectionItems behavior by constructing the DOM correctly for the "details page" path 
        // or just mocking the function if we could, but it's internal.
        // Let's construct a DOM that matches the "details page" selector since we can pass a mock doc.
        
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Software Engineer</span>
                <span aria-hidden="true">Tech Corp</span>
                <span aria-hidden="true">2020 - Present · 3 yrs</span>
                <span aria-hidden="true">New York</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Built cool stuff.</span>
                </div>
            </div>
        `;
        
        const results = content.getExperienceFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].title).toBe('Software Engineer');
        expect(results[0].company).toBe('Tech Corp');
        expect(results[0].description).toBe('Built cool stuff.');
    });

    test('getEducationFromDoc extracts items', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">University of Tech</span>
                <span aria-hidden="true">Bachelor of Science</span>
                <span aria-hidden="true">2016 - 2020</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Graduated with honors.</span>
                </div>
            </div>
        `;
        
        const results = content.getEducationFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].school).toBe('University of Tech');
        expect(results[0].degree).toBe('Bachelor of Science');
        expect(results[0].date_range).toBe('2016 - 2020');
        expect(results[0].description).toBe('Graduated with honors.');
    });

    test('getSkillsFromDoc extracts skills', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">JavaScript</span>
            </div>
            <div class="pvs-list__paged-list-item">
                <div class="display-flex align-items-center mr1 hoverable-link-text">
                    <span aria-hidden="true">Rust</span>
                </div>
            </div>
        `;
        
        const results = content.getSkillsFromDoc(mockDoc);
        expect(results).toEqual(['JavaScript', 'Rust']);
    });

    test('getLanguagesFromDoc extracts languages', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">English</span>
                <span aria-hidden="true">Native or bilingual proficiency</span>
            </div>
        `;
        
        const results = content.getLanguagesFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].name).toBe('English');
        expect(results[0].proficiency).toBe('Native or bilingual proficiency');
    });
});
