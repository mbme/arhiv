module.exports = {
  preflight: {
    enableAll: true,
  },
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
  plugins: [
    require('windicss/plugin/forms'),
    require('windicss/plugin/line-clamp'),
    require('windicss/plugin/typography'),
  ],
};
