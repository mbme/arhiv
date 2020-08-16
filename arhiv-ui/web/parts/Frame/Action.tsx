import * as React from 'react'
import {
  Button,
  Link,
} from '@v/web-platform'
import { SimpleLocation } from '@v/web-utils'

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
  if (action.type === 'location') {
    return (
      <Link
        to={action.to}
      >
        {action.children}
      </Link>
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
