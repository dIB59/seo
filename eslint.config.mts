import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import { defineConfig } from "eslint/config";
import youMightNotNeedAnEffect from "eslint-plugin-react-you-might-not-need-an-effect";

export default defineConfig([
  {
    ignores: ["src/components/ui/**/*"],
  },
  { files: ["**/*.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"], plugins: { js }, extends: ["js/recommended"], languageOptions: { globals: globals.browser } },
  tseslint.configs.recommended,
  pluginReact.configs.flat.recommended,
  pluginReact.configs.flat["jsx-runtime"],
  {
    plugins: {
      "react-you-might-not-need-an-effect": youMightNotNeedAnEffect,
    },
    rules: {
      ...youMightNotNeedAnEffect.configs.recommended.rules,
    },
  },
  {
    settings: {
      react: {
        version: "19.2.0",
      },
    },
    rules: {
      "react/react-in-jsx-scope": "off",
      "react/jsx-uses-react": "off",
    },
  },
]);
