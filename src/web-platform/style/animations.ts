import { keyframes } from './style'

export const animations = {
  pulse: keyframes({
    '0%': {
      opacity: 0.7,
    },

    '50%': {
      opacity: 1,
    },

    '100%': {
      opacity: 0.7,
    },
  }),

  spin: keyframes({
    from: {
      transform: 'rotate(0deg)',
    },

    to: {
      transform: 'rotate(359deg)',
    },
  }),
}
