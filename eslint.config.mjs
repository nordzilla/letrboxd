import jsdoc from "eslint-plugin-jsdoc";

export default [
  {
    plugins: {
      jsdoc,
    },
    ignores: ["site/generated/**/*"],
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: "module"
    },
    rules: {
      semi: "error",
      "prefer-const": "error",
      quotes: ["error", "double"],
      indent: ["error", 2, { SwitchCase: 1 }], // Adjust case indentation level
      curly: ["error", "all"],
      "brace-style": ["error", "1tbs", { allowSingleLine: false }],
      "object-curly-newline": ["error", {
        "multiline": true,
        "consistent": true
      }],
      "object-property-newline": ["error", { "allowAllPropertiesOnSameLine": true }],
      
      // JSDoc-specific rules
      "jsdoc/require-param": "warn", // Warns if function parameters are missing @param annotations
      "jsdoc/require-param-description": "warn", // Ensures descriptions are provided for parameters
      "jsdoc/require-param-type": "warn", // Ensures types are specified for parameters
    },
    settings: {
      jsdoc: {
        mode: "typescript", // Use TypeScript mode if you're working with TypeScript annotations
      },
    },
  }
];
