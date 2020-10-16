import * as React from 'react'
import { Button, StyleArg } from '@v/web-platform'
import { RouterContext, SimpleLocation } from '@v/web-utils'

const $action: StyleArg = {
  textTransform: 'uppercase',
  width: '100%',
}

type Action = {
  type: 'location',
  to: SimpleLocation,
  replace?: boolean
  children: string,
} | {
  type: 'action',
  onClick(): void,
  children: string,
  disabled?: boolean,
}

export function Action(action: Action) {
  const router = RouterContext.use()

  const onClick = () => {
    if (action.type === 'location') {
      if (action.replace) {
        router.replace(action.to)
      } else {
        router.push(action.to)
      }
    }

    if (action.type === 'action') {
      action.onClick()
    }
  }

  return (
    <Button
      onClick={onClick}
      disabled={'disabled' in action ? action.disabled : false}
      $style={$action}
    >
      {action.children}
    </Button>
  )
}
