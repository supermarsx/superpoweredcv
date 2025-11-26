const globals = require("globals");
const js = require("@eslint/js");

module.exports = [
    js.configs.recommended,
    {
        languageOptions: {
            ecmaVersion: 12,
            sourceType: "module",
            globals: {
                ...globals.browser,
                ...globals.webextensions,
                ...globals.jest,
                ...globals.es2021,
                chrome: "readonly",
                module: "readonly"
            }
        },
        rules: {
            "no-unused-vars": "warn",
            "no-console": "off",
            "no-undef": "warn"
        }
    }
];
