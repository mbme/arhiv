/* eslint-env node */
import { Config } from 'tailwindcss';

const config: Config = {
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

export default config;
