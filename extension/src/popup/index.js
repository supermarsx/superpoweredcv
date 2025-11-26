/**
 * @file popup/index.js
 * @description Popup script for the extension. Handles user interaction.
 */

document.addEventListener('DOMContentLoaded', () => {
    const grabBtn = document.getElementById('grabBtn');
    const statusDiv = document.getElementById('status');
    const previewDiv = document.getElementById('preview');

    if (grabBtn) {
        grabBtn.addEventListener('click', async () => {
            await handleGrabClick(statusDiv, previewDiv);
        });
    }
});

/**
 * Handles the click event on the Grab button.
 * @param {HTMLElement} statusDiv - The status display element.
 * @param {HTMLElement} previewDiv - The preview display element.
 */
async function handleGrabClick(statusDiv, previewDiv) {
    try {
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        
        if (!tab || !tab.url.includes('linkedin.com/in/')) {
            updateStatus(statusDiv, 'Not a LinkedIn profile page.');
            return;
        }

        updateStatus(statusDiv, 'Scraping...');
        if (previewDiv) previewDiv.innerHTML = ''; // Clear previous preview
        Logger.info('Sending scrape command to tab', tab.id);

        const response = await chrome.tabs.sendMessage(tab.id, { action: 'scrape' });
        
        if (response && response.data) {
            Logger.info('Scrape successful', response.data);
            
            // Render preview using our "templating engine"
            if (previewDiv) {
                previewDiv.innerHTML = renderProfileTemplate(response.data);
            }

            downloadJSON(response.data, `profile_${response.data.name || 'unknown'}.json`);
            updateStatus(statusDiv, 'Done!');
        } else if (response && response.error) {
            Logger.error('Scrape error from content script', response.error);
            updateStatus(statusDiv, 'Failed: ' + response.error);
        } else {
            Logger.error('Scrape failed, no data');
            updateStatus(statusDiv, 'Failed to scrape.');
        }
    } catch (error) {
        Logger.error('Popup error', error);
        updateStatus(statusDiv, 'Error: ' + error.message);
    }
}

/**
 * Simple templating engine to render profile preview.
 * @param {Object} data - The profile data.
 * @returns {string} HTML string.
 */
function renderProfileTemplate(data) {
    const { name, headline, location, experience, education } = data;
    
    const expCount = experience ? experience.length : 0;
    const eduCount = education ? education.length : 0;

    return `
        <div class="preview-item">
            <span class="preview-key">NAME:</span> ${escapeHtml(name)}
        </div>
        <div class="preview-item">
            <span class="preview-key">HEADLINE:</span> ${escapeHtml(headline).substring(0, 50)}...
        </div>
        <div class="preview-item">
            <span class="preview-key">LOC:</span> ${escapeHtml(location)}
        </div>
        <div class="preview-item">
            <span class="preview-key">STATS:</span> ${expCount} EXP / ${eduCount} EDU
        </div>
    `;
}

/**
 * Escapes HTML characters to prevent XSS.
 * @param {string} str - The string to escape.
 * @returns {string} Escaped string.
 */
function escapeHtml(str) {
    if (!str) return '';
    return str
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}


/**
 * Updates the status text.
 * @param {HTMLElement} element - The status element.
 * @param {string} text - The text to display.
 */
function updateStatus(element, text) {
    if (element) {
        element.textContent = text;
    }
}

/**
 * Triggers a download of the scraped data as a JSON file.
 * @param {Object} data - The profile data to download.
 * @param {string} filename - The name of the file to save.
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
