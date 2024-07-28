/* eslint-env node */

module.exports = {
  plugins: ['@typescript-eslint', 'react-hooks'],
  extends: [
    'eslint:recommended',
    'plugin:react-hooks/recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-type-checked',
    'plugin:@typescript-eslint/strict',
    'plugin:@typescript-eslint/strict-type-checked',
    'prettier',
  ],
  env: {
    'browser': true,
    'es2021': true,
  },
  parser: '@typescript-eslint/parser',
  parserOptions: {
    'ecmaVersion': 12,
    'sourceType': 'module',
  },
  rules: {
    'linebreak-style': ['error', 'unix'],
    'no-prototype-builtins': 0,
    '@typescript-eslint/no-unused-vars': ['error', { 'argsIgnorePattern': '^_' }],
    '@typescript-eslint/no-non-null-assertion': 0,
    '@typescript-eslint/restrict-template-expressions': ['error', { allowNumber: true }],
  },
  root: true,
};
