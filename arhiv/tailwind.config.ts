/* eslint-env node */
import { Config } from 'tailwindcss';

const config: Config = {
  content: ['./src/ui/**/*.tsx'],
  darkMode: 'selector',
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
};

export default config;
