export const theme = {
  spacing: {
    fine: '0.25rem',
    small: '0.5rem',
    medium: '1.25rem',
    large: '2rem',
    xlarge: '3.25rem',
    xxlarge: '5.25rem',
  },
  border: {
    default: '1px solid var(--color-secondary)',
    active: '1px solid var(--color-primary)',
    invisible: '1px solid transparent',
  },
  boxShadow: {
    default: '0 2px 4px rgba(0,0,0,.2)',
  },
  fontSize: {
    fine: '0.75rem',
    small: '0.875rem',
    medium: '1rem',
    large: '1.777rem',
    xlarge: '2.369rem',
    xxlarge: '3.158rem',
  },
  zIndex: {
    modal: 10,
  },
  breakpoints: {
    sm: '768px',
    md: '1024px',
    lg: '1366px',
  },
  animations: {
    pulse: {
      '0%': {
        opacity: 0.7,
      },

      '50%': {
        opacity: 1,
      },

      '100%': {
        opacity: 0.7,
      },
    },

    spin: {
      from: {
        transform: 'rotate(0deg)',
      },

      to: {
        transform: 'rotate(359deg)',
      },
    },
  },
}
