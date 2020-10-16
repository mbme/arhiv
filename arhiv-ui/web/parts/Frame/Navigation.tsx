import * as React from 'react'
import {
  Column,
  Link,
  StyleArg,
} from '@v/web-platform'

const getStyles = (isActive?: boolean): StyleArg[] => [
  {
    width: '100%',
    display: 'flex',
    justifyContent: 'center',
    textTransform: 'uppercase',
    py: 'fine',
  },
  isActive && {
    color: 'var(--color-link)',
    '&:hover': {
      color: 'var(--color-link)',
    },
  },
]

export function Navigation() {
  return (
    <Column>
      <Link
        to={{ path: '/' }}
        $styles={getStyles(true)}
        clean
      >
        Notes
      </Link>

      <Link
        to={{ path: '/' }}
        $styles={getStyles()}
        clean
      >
        Contacts
      </Link>

      <Link
        to={{ path: '/' }}
        $styles={getStyles()}
        clean
      >
        Movies
      </Link>
    </Column>
  )
}
