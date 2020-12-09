import * as React from 'react'
import {
  Column,
  Link,
  StyleArg,
} from '@v/web-platform'
import { MODULES } from '../../api'

const $style: StyleArg =  {
  width: '100%',
  display: 'flex',
  justifyContent: 'center',
  textTransform: 'uppercase',
  py: 'fine',
}

export function Navigation() {
  const links = Object.keys(MODULES).map(module => (
    <Link
      key={module}
      to={`/catalog/${module}`}
      $style={$style}
      clean
    >
      {module}
    </Link>
  ))

  return (
    <Column>
      {links}
    </Column>
  )
}
