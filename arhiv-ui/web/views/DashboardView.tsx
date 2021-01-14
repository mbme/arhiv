import * as React from 'react'
import { Box, Link, StyleArg } from '@v/web-platform'
import { FrameTitle } from '../parts'
import { SCHEMA } from '../api'

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

  maxHeight: '100%',

  flex: '1 1 auto',
  pr: 'medium',
}

export function DashboardView() {
  const links = SCHEMA.modules.map(module => (
    <Link
      key={module.documentType}
      to={`/catalog/${module.documentType}`}
      $style={$link}
      clean
    >
      {module.documentType}
    </Link>
  ))

  return (
    <>
      <FrameTitle>
        Dashboard
      </FrameTitle>

      <Box $style={$container}>
        {links}
      </Box>
    </>
  )
}
