/* eslint-env node */
const plugin = require('tailwindcss/plugin');

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
  plugins: [
    require('@tailwindcss/typography'),
    require('@tailwindcss/forms'),
    require('@tailwindcss/line-clamp'),
    plugin(function ({ addVariant }) {
      addVariant('is-active', '&[data-is-active]');
    }),
  ],
};
