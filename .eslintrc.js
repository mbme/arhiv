/* eslint-env node */

module.exports = {
  plugins: ['@typescript-eslint'],
  extends: [
    'eslint:recommended',
    'prettier',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
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
  },
};
