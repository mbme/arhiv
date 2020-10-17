import * as React from 'react'
import {
  Column,
  StyleArg,
} from '@v/web-platform'

const $actionContainer: StyleArg = {
  '&>*': {
    mb: 'small',
  },
}

interface IProps {
  actions: React.ReactNode,
}

export function Actions({ actions }: IProps) {
  return (
    <Column
      $style={$actionContainer}
    >
      {actions}
    </Column>
  )
}
