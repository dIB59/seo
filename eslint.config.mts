import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import { defineConfig } from "eslint/config";
import youMightNotNeedAnEffect from "eslint-plugin-react-you-might-not-need-an-effect";

const generatedContractImportMessage =
  "Import generated contracts through a domain API module in src/api instead of importing generated surfaces directly.";

const restrictedGeneratedContractPaths = ["@/src/bindings", "@/src/bindings.ts"];

const restrictedGeneratedContractPatterns = [
  "@/src/gen/**",
  "@/src/generated/**",
  "@/src/**/__generated__/**",
];

export default defineConfig([
  {
    ignores: [
      "src/components/ui/**/*",
      "src/bindings.ts",
      "src-tauri/**/*",
      "node_modules/**/*",
      ".next/**/*",
      "out/**/*",
      "target/**/*",
    ],
  },
  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
    plugins: { js },
    extends: ["js/recommended"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.mocha,
        describe: "readonly",
        it: "readonly",
        expect: "readonly",
        $: "readonly",
        browser: "readonly",
      },
    },
  },
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
  {
    files: ["src/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: restrictedGeneratedContractPaths.map((name) => ({
            name,
            message: generatedContractImportMessage,
          })),
          patterns: restrictedGeneratedContractPatterns.map((group) => ({
            group: [group],
            message: generatedContractImportMessage,
          })),
        },
      ],
    },
  },
  {
    files: ["src/api/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": "off",
    },
  },
]);
