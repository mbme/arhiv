import * as React from 'react'
import { Button, StyleArg } from '@v/web-platform'
import { ActionsRegistry } from './context'

const $action: StyleArg = {
  textTransform: 'uppercase',
  width: '100%',
  mb: 'medium',
}

export function ActionsRenderer() {
  const values = ActionsRegistry.useValues()

  const items = values
    .flatMap(({ item: actions }) => actions)
    .map((action, i) => (
      <Button
        key={i}
        onClick={action.onClick}
        disabled={action.disabled}
        $style={$action}
      >
        {action.children}
      </Button>
    ))

  return (
    <>
      {items}
    </>
  )
}
