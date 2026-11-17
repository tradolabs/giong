import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["dist/**", "coverage/**", "**/README*"]),
  {
    files: ["**/*.{js,jsx,ts,tsx}"],
    extends: [js.configs.recommended, tseslint.configs.recommended],
    languageOptions: {
      globals: globals.browser,
    },
  },
  {
    files: ["eslint.config.js", "vite.config.ts"],
    languageOptions: {
      globals: globals.node,
    },
  },
]);
