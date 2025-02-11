import globals from 'globals';
import pluginJs from '@eslint/js';
import tseslint from 'typescript-eslint';
import pluginReact from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import eslintConfigPrettier from 'eslint-config-prettier';

export default tseslint.config([
  { files: ['**/*.{js,ts,jsx,tsx}', 'eslint.config.js'] },
  { languageOptions: { globals: globals.browser } },
  {
    ignores: ['./arhiv/public/'],
  },
  pluginJs.configs.recommended,
  {
    rules: {
      'linebreak-style': ['error', 'unix'],
      'no-prototype-builtins': 0,
    },
  },
  tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'error',
        { 'argsIgnorePattern': '^_', 'caughtErrorsIgnorePattern': '^_' },
      ],
      '@typescript-eslint/no-non-null-assertion': 0,
      '@typescript-eslint/restrict-template-expressions': ['error', { allowNumber: true }],
      '@typescript-eslint/no-unnecessary-type-parameters': 0,
    },
  },
  pluginReact.configs.flat.recommended,
  pluginReact.configs.flat['jsx-runtime'],
  {
    settings: {
      react: {
        version: 'detect',
      },
    },
  },
  {
    plugins: {
      'react-hooks': reactHooks,
    },
    rules: { ...reactHooks.configs.recommended.rules },
  },
  eslintConfigPrettier,
]);
