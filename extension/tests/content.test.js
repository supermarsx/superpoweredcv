const content = require('../src/content/index.js');

describe('Content Script', () => {
    test('getText returns empty string if document is undefined', () => {
        expect(content.getText('h1')).toBe('');
    });

    // We can mock document for more tests
    test('getText returns text content', () => {
        // Create a dummy element
        const dummyEl = document.createElement('div');
        dummyEl.innerText = '  Hello World  ';
        
        // Spy on querySelector
        jest.spyOn(document, 'querySelector').mockReturnValue(dummyEl);
        
        expect(content.getText('h1')).toBe('Hello World');
        
        // Restore mock
        document.querySelector.mockRestore();
    });
});
