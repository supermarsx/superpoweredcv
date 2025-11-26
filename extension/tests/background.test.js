// Mock chrome API BEFORE requiring the background script
global.chrome = {
    runtime: {
        onMessage: {
            addListener: jest.fn()
        },
        sendMessage: jest.fn(),
        lastError: null
    },
    tabs: {
        get: jest.fn(),
        update: jest.fn(),
        sendMessage: jest.fn(),
        onUpdated: {
            addListener: jest.fn(),
            removeListener: jest.fn()
        }
    },
    scripting: {
        executeScript: jest.fn()
    }
};

const { handleStartScrape, ensureContentScriptReady, sendMessageToTab } = require('../src/background/index.js');

describe('Background Script', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });

    test('ensureContentScriptReady returns true if ping succeeds', async () => {
        chrome.tabs.sendMessage.mockImplementation((tabId, msg, cb) => {
            cb({ status: 'alive' });
        });

        const result = await ensureContentScriptReady(123);
        expect(result).toBe(true);
        expect(chrome.tabs.sendMessage).toHaveBeenCalled();
    });

    test('ensureContentScriptReady injects script if ping fails', async () => {
        // Fail the first 10 pings (MAX_ATTEMPTS)
        let callCount = 0;
        chrome.tabs.sendMessage.mockImplementation((tabId, msg, cb) => {
            callCount++;
            if (callCount <= 10) {
                chrome.runtime.lastError = { message: 'Receiving end does not exist' };
                cb(null);
            } else {
                chrome.runtime.lastError = null;
                cb({ status: 'alive' });
            }
        });

        const result = await ensureContentScriptReady(123);
        
        expect(chrome.scripting.executeScript).toHaveBeenCalledWith({
            target: { tabId: 123 },
            files: ['src/content/index.js']
        });
        expect(result).toBe(true);
    });
});
