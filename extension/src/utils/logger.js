/**
 * Logger utility for the extension.
 * Allows for conditional logging based on environment (dev/prod).
 */
const Logger = {
    debug: (...args) => {
        console.log('[SuperpoweredCV DEBUG]:', ...args);
    },
    error: (...args) => {
        console.error('[SuperpoweredCV ERROR]:', ...args);
    },
    info: (...args) => {
        console.info('[SuperpoweredCV INFO]:', ...args);
    }
};

if (typeof module !== 'undefined') {
    module.exports = Logger;
} else {
    window.Logger = Logger;
}
