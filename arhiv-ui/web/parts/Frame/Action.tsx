import * as React from 'react'
import {
  Button,
} from '@v/web-platform'
import { RouterContext, SimpleLocation } from '@v/web-utils'

type Action = {
  type: 'location',
  to: SimpleLocation,
  children: string,
} | {
  type: 'action',
  onClick(): void,
  children: string,
  disabled?: boolean,
}

export function Action(action: Action) {
  const router = RouterContext.use()

  if (action.type === 'location') {
    return (
      <Button
        onClick={() => router.push(action.to)}
      >
        {action.children}
      </Button>
    )
  }

  if (action.type === 'action') {
    return (
      <Button
        onClick={action.onClick}
        disabled={action.disabled}
      >
        {action.children}
      </Button>
    )
  }

  return null
}
