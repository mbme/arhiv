export const theme = {
  spacing: {
    fine: '0.25rem',
    small: '0.5rem',
    medium: '1.25rem',
    large: '2rem',
    xlarge: '3rem',
  },
  border: {
    default: '1px solid rgba(0,0,0,.09)',
  },
  boxShadow: {
    default: '0 2px 4px rgba(0,0,0,.2)',
  },
  color: {
    primary: '#FF553C',
    secondary: '#5E5A57',
    light: '#ffffff',
    text: '#333333',
    heading: '#000000',
    link: '#FDAF3C',
    bg0: '#ffffff',
    bg1: '#f5f5f5',
    bg2: '#F9F9FA',
    bgOverlay: 'rgba(255,255,255, .65)',
    danger: 'red',
  },
  fontFamily: {
    base: [
      '-apple-system', 'BlinkMacSystemFont', // Safari Mac/iOS, Chrome
      '"Segoe UI"', 'Roboto', 'Oxygen', // Windows, Android, KDE
      'Ubuntu', 'Cantarell', '"Fira Sans"', // Ubuntu, Gnome, Firefox OS
      '"Droid Sans"', '"Helvetica Neue"', 'sans-serif', // Old Android
    ].join(', '),
    mono: [
      'SFMono-Regular',
      'Menlo',
      'Monaco',
      'Consolas',
      '"Liberation Mono"',
      '"Courier New"',
      'monospace',
    ].join(', '),
  },
  fontSize: {
    fine: '0.75rem',
    small: '0.875rem',
    medium: '1rem',
    large: '1.777rem',
    xlarge: '2.369rem',
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
