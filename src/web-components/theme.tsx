import * as React from 'react'
import {
  style,
} from 'typestyle'

const theme = {
  spacing: {
    fine: '0.25rem',
    small: '0.5rem',
    medium: '1.25rem',
    large: '2rem',
  },
  border: '1px solid rgba(0,0,0,.09)',
  boxShadow: '0 2px 4px rgba(0,0,0,.2)',
  color: {
    primary: '#FF553C',
    primaryLighter: '#FF7B67',
    secondary: '#5E5A57',
    light: '#ffffff',
    dark: '#484642',
    text: '#333333',
    heading: '#000000',
    link: '#FDAF3C',
    bg: '#ffffff',
    bgDarker: '#f5f5f5',
    backdrop: 'rgba(255,255,255, .65)',
  },
  fontFamily: {
    base: [
      '-apple-system', 'BlinkMacSystemFont', // Safari Mac/iOS, Chrome
      '"Segoe UI"', 'Roboto', 'Oxygen', // Windows, Android, KDE
      'Ubuntu', 'Cantarell', '"Fira Sans"', // Ubuntu, Gnome, Firefox OS
      '"Droid Sans"', '"Helvetica Neue"', 'sans-serif', // Old Android
    ].join(', '),
    mono: 'monospace, monospace',
  },
  fontSize: {
    fine: '0.75rem',
    small: '0.875rem',
    medium: '1rem',
    large: '1.777rem',
    xlarge: '2.369rem',
  },
  maxWidth: '35rem',
  zIndex: {
    modal: 10,
  },
}

export default theme

export const examples = {
  Colors: (
    <div>
      {Object.entries(theme.color).map(([name, value]) => (
        <div
          key={name}
          className={style({
            height: '3rem',
            backgroundColor: value,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          })}
        >
          {name}
        </div>
      ))}
    </div>
  ),
}
