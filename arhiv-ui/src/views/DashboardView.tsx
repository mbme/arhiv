import * as React from 'react'
import { Column, Link, StyleArg } from '@v/web-platform'
import { FrameTitle } from '../parts'
import { DataManagerContext } from '../data-manager'

const $link: StyleArg = {
  textTransform: 'uppercase',
  bgColor: 'rgb(224 255 255 / 92%)',
  fontSize: 'xlarge',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  minWidth: '16rem',
  mb: 'medium',
}

export function DashboardView() {
  const dataManager = DataManagerContext.use()

  const links = dataManager.getModules().map(module => (
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

      <Column>
        {links}
      </Column>
    </>
  )
}
