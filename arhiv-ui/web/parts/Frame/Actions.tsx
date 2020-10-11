import * as React from 'react'
import {
  Column,
  StyleArg,
} from '@v/web-platform'

const $actionContainer: StyleArg = {
  '&>*': {
    my: 'small',
  },
}

interface IProps {
  actions: React.ReactNode,
}

export function Actions({ actions }: IProps) {
  return (
    <Column
      pl="small"
      $style={$actionContainer}
    >
      {actions}
    </Column>
  )
}
