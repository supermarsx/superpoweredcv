# SuperpoweredCV Profile Grabber

This is a browser extension to scrape LinkedIn profiles and export them as JSON for SuperpoweredCV analysis. It features a brutalist, high-contrast design and supports all major browsers.

## Installation Instructions

### Google Chrome / Chromium (Brave, Vivaldi, Opera)
1.  Download the `superpoweredcv-chrome.zip` from the `dist` folder (or build it yourself).
2.  Unzip the file.
3.  Open Chrome and navigate to `chrome://extensions`.
4.  Enable **Developer mode** in the top right corner.
5.  Click **Load unpacked**.
6.  Select the unzipped folder (or the `extension` folder in this repo).

### Mozilla Firefox
1.  Download the `superpoweredcv-firefox.zip` from the `dist` folder.
2.  Open Firefox and navigate to `about:debugging#/runtime/this-firefox`.
3.  Click **Load Temporary Add-on...**.
4.  Select the `manifest.json` file inside the unzipped folder (or `manifests/firefox.json` if loading from source, though you may need to rename it or use the build script).
    *   *Note: For permanent installation, the extension needs to be signed by Mozilla.*

### Microsoft Edge
1.  Download the `superpoweredcv-edge.zip` from the `dist` folder.
2.  Unzip the file.
3.  Open Edge and navigate to `edge://extensions`.
4.  Enable **Developer mode** (toggle on the left or bottom).
5.  Click **Load unpacked**.
6.  Select the unzipped folder.

### Apple Safari
1.  Safari requires converting the Web Extension to a Safari App Extension.
2.  Ensure you have Xcode installed.
3.  Run: `xcrun safari-web-extension-converter dist/superpoweredcv-safari.zip`
4.  Follow the Xcode prompts to build and run the app.
5.  Enable the extension in Safari Preferences > Extensions.

## Extension Structure & Flow

The extension is built using the WebExtensions API (Manifest V3).

### Directory Structure
*   `src/`: Source code.
    *   `content/`: Scripts that run on the LinkedIn page to scrape data.
    *   `popup/`: The extension's UI (HTML, CSS, JS).
    *   `utils/`: Shared utilities (Logger).
*   `manifests/`: Browser-specific manifest files.
*   `scripts/`: Build and packaging scripts.
*   `tests/`: Unit tests.

### Data Flow
1.  **Initialization**: The user clicks the extension icon while on a LinkedIn profile.
2.  **Popup Render**: `src/popup/index.html` is displayed, styled by `src/popup/styles.css`.
3.  **User Action**: User clicks "INITIATE_DUMP".
4.  **Message Passing**: `src/popup/index.js` sends a `scrape` message to the active tab via `chrome.tabs.sendMessage`.
5.  **Scraping**: `src/content/index.js` receives the message. It queries the DOM for specific selectors (Name, Headline, About, Experience, Education, Skills).
6.  **Response**: The content script returns a JSON object with the scraped data.
7.  **Preview & Export**:
    *   The popup receives the data.
    *   A simple templating engine renders a preview of the data in the popup.
    *   The JSON data is automatically downloaded as a file (`profile_Name.json`).

## Development

### Prerequisites
*   Node.js & npm
*   PowerShell (Windows) or Bash (Unix)

### Setup
```bash
cd extension
npm install
```

### Linting & Testing
```bash
npm run lint
npm test
```

### Building
To create zip packages for all browsers:
```bash
# Windows
.\scripts\package_extension.ps1

# Unix
./scripts/package_extension.sh
```
The output will be in the `dist/` directory.
