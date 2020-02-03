module.exports = {
  "env": {
    "browser": true,
    "es6": true,
    "node": true
  },
  "extends": [
    "airbnb-typescript",
    "airbnb/hooks",
    "plugin:@typescript-eslint/eslint-recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    "plugin:import/typescript"
  ],
  "parser": "@typescript-eslint/parser",
  "parserOptions": {
    "project": "tsconfig.json",
    "sourceType": "module"
  },
  "plugins": [
    "@typescript-eslint"
  ],
  settings: {
    'import/resolver': {
      'typescript': {},
    },
  },
  rules: {
    "no-underscore-dangle": "off",
    "arrow-parens": [
      "error",
      "as-needed",
      { "requireForBlockBody": true },
    ],
    "no-restricted-syntax": [
      "error",
      "WithStatement",
    ],
    "object-curly-newline": [
      "error",
      {
        "consistent": true,
      },
    ],
    "quote-props": "off",
    "no-continue": "off",
    "curly": [
      "error",
      "all",
    ],
    "lines-between-class-members": "off",
    "prefer-template": "off",
    "max-classes-per-file": "off",
    "spaced-comment": ["error", "always", { "markers": ["/"] }],
    "no-await-in-loop": "off",
    "max-len": [
      "error",
      120
    ],
    "class-methods-use-this": "off",
    "prefer-destructuring": "off",
    "prefer-arrow-callback": "off",

    // imports
    "import/prefer-default-export": "off",
    "import/no-default-export": "error",

    // react
    "react/jsx-props-no-spreading": "off",
    "react/state-in-constructor": "off",
    "react/sort-comp": "off",
    "react/jsx-one-expression-per-line": "off",
    "react/prop-types": "off",

    // typescript-eslint
    "@typescript-eslint/adjacent-overload-signatures": "error",
    "@typescript-eslint/array-type": "off",
    "@typescript-eslint/await-thenable": "error",
    "@typescript-eslint/ban-ts-ignore": "error",
    "@typescript-eslint/ban-types": "error",
    "@typescript-eslint/class-name-casing": "error",
    "@typescript-eslint/consistent-type-assertions": "error",
    "@typescript-eslint/consistent-type-definitions": "error",
    "@typescript-eslint/interface-name-prefix": [
      "error",
      {
        "prefixWithI": "always",
      },
    ],
    "@typescript-eslint/explicit-member-accessibility": [
      "off",
      {
        "accessibility": "explicit"
      }
    ],
    "@typescript-eslint/explicit-function-return-type": "off",
    "@typescript-eslint/indent": [
      "error",
      2,
    ],
    "@typescript-eslint/member-delimiter-style": [
      "off",
      {
        "multiline": {
          "delimiter": "none",
          "requireLast": true
        },
        "singleline": {
          "delimiter": "semi",
          "requireLast": false
        }
      }
    ],
    "@typescript-eslint/member-ordering": "off",
    "@typescript-eslint/no-empty-function": "error",
    "@typescript-eslint/no-empty-interface": "error",
    "@typescript-eslint/no-explicit-any": "off",
    "@typescript-eslint/no-extraneous-class": "error",
    "@typescript-eslint/no-floating-promises": "error",
    "@typescript-eslint/no-for-in-array": "error",
    "@typescript-eslint/no-inferrable-types": "error",
    "@typescript-eslint/no-misused-new": "error",
    "@typescript-eslint/no-namespace": "error",
    "@typescript-eslint/no-non-null-assertion": "off",
    "@typescript-eslint/no-parameter-properties": "off",
    "@typescript-eslint/no-require-imports": "error",
    "@typescript-eslint/no-this-alias": "error",
    "@typescript-eslint/no-unnecessary-qualifier": "error",
    "@typescript-eslint/no-unnecessary-type-arguments": "error",
    "@typescript-eslint/no-unnecessary-type-assertion": "error",
    "@typescript-eslint/no-use-before-define": "off",
    "@typescript-eslint/no-var-requires": "error",
    "@typescript-eslint/prefer-for-of": "error",
    "@typescript-eslint/prefer-function-type": "error",
    "@typescript-eslint/prefer-namespace-keyword": "error",
    "@typescript-eslint/promise-function-async": "off",
    "@typescript-eslint/quotes": [
      "error",
      "single",
      {
        "avoidEscape": true
      }
    ],
    "@typescript-eslint/require-await": "error",
    "@typescript-eslint/restrict-plus-operands": "error",
    "@typescript-eslint/semi": [
      "error",
      "never"
    ],
    "@typescript-eslint/strict-boolean-expressions": "off",
    "@typescript-eslint/triple-slash-reference": "error",
    "@typescript-eslint/type-annotation-spacing": "error",
    "@typescript-eslint/unbound-method": "error",
    "@typescript-eslint/unified-signatures": "error",
  }
};
