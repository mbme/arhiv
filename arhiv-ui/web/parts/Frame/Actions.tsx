import * as React from 'react'
import {
  Column,
  StyleArg,
  useFocusedRegion,
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
  const isActive = useFocusedRegion()

  return (
    <Column
      pl="small"
      $style={$actionContainer}
      bgColor={isActive ? 'var(--color-bg-highlight)' : undefined}
    >
      {actions}
    </Column>
  )
}
