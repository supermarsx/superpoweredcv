# SuperpoweredCV Browser Extension Specification

## Overview
The SuperpoweredCV Browser Extension is a tool designed to scrape LinkedIn profile data directly from the browser and export it as a JSON file. This data is then used by the SuperpoweredCV core system to generate tailored resumes and cover letters.

## Architecture
The extension follows the standard WebExtensions API and supports Chrome, Firefox, Edge, and Safari.

### Components
1.  **Manifest**: Defines the extension's metadata, permissions, and entry points.
    *   `manifests/chrome.json`: For Chrome and Chromium-based browsers (Manifest V3).
    *   `manifests/firefox.json`: For Firefox (Manifest V3 with Gecko specifics).
    *   `manifests/edge.json`: For Microsoft Edge.
    *   `manifests/safari.json`: For Safari.
2.  **Popup**: The user interface.
    *   `src/popup/index.html`: The HTML structure.
    *   `src/popup/index.js`: Handles user interactions (button clicks) and communication with the content script.
    *   `src/popup/styles.css`: Styling for the popup.
3.  **Content Script**: Runs in the context of the LinkedIn page.
    *   `src/content/index.js`: Scrapes data from the DOM (Name, Headline, About, Experience, Education, Skills).
4.  **Utils**: Shared utilities.
    *   `src/utils/logger.js`: Logging utility for debugging.

## Data Flow
1.  User navigates to a LinkedIn profile.
2.  User clicks the extension icon to open the Popup.
3.  User clicks "INITIATE_DUMP".
4.  Popup sends a `scrape` message to the active tab.
5.  Content Script receives the message, scrapes the DOM, and returns a JSON object.
6.  Popup receives the data and triggers a file download (`profile_<name>.json`).

## Data Structure
The scraped JSON follows this schema:
```json
{
  "name": "String",
  "headline": "String",
  "location": "String",
  "about": "String",
  "experience": [
    {
      "title": "String",
      "company": "String",
      "date_range": "String",
      "location": "String",
      "description": "String",
      "skills": ["String"]
    }
  ],
  "education": [
    {
      "school": "String",
      "degree": "String"
    }
  ],
  "languages": [
    {
      "name": "String",
      "proficiency": "String"
    }
  ],
  "skills": ["String"],
  "url": "String"
}
```

## Browser Support
*   **Google Chrome**: Fully supported (Manifest V3).
*   **Mozilla Firefox**: Fully supported (Manifest V3).
*   **Microsoft Edge**: Fully supported (Manifest V3).
*   **Apple Safari**: Supported via Web Extension converter.

## Development
*   **Linting**: ESLint is used for code quality.
*   **Testing**: Jest is used for unit testing logic.
*   **Build**: A packaging script bundles the extension for each browser.
