import * as React from 'react'
import { Icon, StyleArg } from '@v/web-platform'
import { Procedure } from '@v/utils'

const $icon: StyleArg = {
  color: 'secondary',
}

interface IProps {
  onClick: Procedure
}

export function CloseIcon({ onClick }: IProps) {
  return (
    <Icon
      type="x"
      title="close"
      onClick={onClick}
      $style={$icon}
    />
  )
}
