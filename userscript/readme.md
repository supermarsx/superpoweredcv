# SuperpoweredCV Userscript

This is a userscript alternative to the browser extension. It provides the same functionality—scraping LinkedIn profiles and exporting them as JSON—but runs via a userscript manager like Tampermonkey or Violentmonkey.

## Features
*   **Brutalist UI**: Injects a high-contrast "INITIATE_DUMP" button directly onto the LinkedIn profile page.
*   **One-Click Scraping**: Extracts Name, Headline, About, Experience, Education, and Skills.
*   **JSON Export**: Automatically downloads the data as a JSON file compatible with the SuperpoweredCV core.

## Installation

1.  **Install a Userscript Manager**:
    *   [Tampermonkey](https://www.tampermonkey.net/) (Chrome, Firefox, Edge, Safari)
    *   [Violentmonkey](https://violentmonkey.github.io/) (Open Source alternative)

2.  **Install the Script**:
    *   Click on the `linkedin_profile_grabber.user.js` file in this directory.
    *   Click the "Raw" button (if viewing on GitHub) or simply open the file in your browser.
    *   Your userscript manager should detect the file and prompt you to install it.
    *   Click **Install**.

## Usage

1.  Navigate to any LinkedIn profile page (e.g., `https://www.linkedin.com/in/your-profile`).
2.  Look for the **INITIATE_DUMP** button in the bottom-right corner of the screen.
3.  Click the button.
4.  The button text will change to "SCRAPING..." and then "DONE!".
5.  A file named `profile_<Name>.json` will be downloaded to your computer.

## Development

To modify the script:
1.  Open the dashboard of your userscript manager.
2.  Find "SuperpoweredCV Profile Grabber" and click edit.
3.  You can copy-paste changes from `linkedin_profile_grabber.user.js` into the editor.
