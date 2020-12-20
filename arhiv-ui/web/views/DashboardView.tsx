import * as React from 'react'
import { Box, Link, StyleArg } from '@v/web-platform'
import { Frame } from '../parts'
import { MODULES } from '../api'

const $link: StyleArg = {
  textTransform: 'uppercase',
  bgColor: 'rgb(224 255 255 / 92%)',
  fontSize: 'xlarge',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
}

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(16rem, 1fr))',
  gridTemplateRows: 'repeat(auto-fill, minmax(16rem, 1fr))',
  gridGap: '0.8rem',

  flex: '1 1 auto',
  pr: 'medium',
}

export function DashboardView() {
  const links = Object.keys(MODULES).map(module => (
    <Link
      key={module}
      to={`/catalog/${module}`}
      $style={$link}
      clean
    >
      {module}
    </Link>
  ))

  return (
    <Frame
      title="Dashboard"
    >
      <Box $style={$container}>
        {links}
      </Box>
    </Frame>
  )
}
