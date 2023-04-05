/* eslint-env node */
module.exports = {
  content: ['./src/ui/**/*.tsx'],
  theme: {
    extend: {
      fontFamily: {
        'sans': ['Noto Sans', 'ui-sans-serif', 'system-ui', 'sans'],
      },
      typography: {
        DEFAULT: {
          css: {
            a: {
              color: null, // use global styles
              textDecoration: 'none',
            },
          },
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [require('@tailwindcss/forms')],
};
