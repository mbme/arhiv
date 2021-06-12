/* eslint-env node */

module.exports = {
  mode: 'jit',
  purge: [
    './templates/**/*.html.tera'
  ],
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {
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
  ],
};
