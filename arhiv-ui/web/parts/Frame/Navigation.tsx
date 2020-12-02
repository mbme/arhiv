import * as React from 'react'
import { useLocation } from '@v/web-utils'
import {
  Column,
  Link,
  StyleArg,
} from '@v/web-platform'
import { MODULES } from '../../api'
import { DOCUMENT_TYPE_QUERY_PARAM } from '../../views/CatalogView'

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
  const location = useLocation()

  const isCatalog = location.path === '/documents'
  const activeType = location.params.find(param => param.name === DOCUMENT_TYPE_QUERY_PARAM)

  const links = Object.keys(MODULES).map(module => (
    <Link
      key={module}
      to={{ path: '/documents', params: [{ name: DOCUMENT_TYPE_QUERY_PARAM, value: module }] }}
      $styles={getStyles(isCatalog && activeType?.value === module)}
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
