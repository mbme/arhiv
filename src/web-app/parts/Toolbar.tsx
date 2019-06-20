import * as React from 'react'
import { theme } from '~/web-components'
import { stylish } from '~/stylish'

const $toolbar = stylish({
  position: 'sticky',
  top: 0,
  backgroundColor: theme.color.bg,
  padding: `${theme.spacing.fine} 0`,
  height: '60px',
  marginBottom: theme.spacing.medium,

  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  flexWrap: 'nowrap',
})

const $container = stylish(
  props => ({
    '& > *': {
      [props.left ? 'marginRight' : 'marginLeft']: '1rem',
    },
  }),
)

interface IProps {
  left?: React.ReactNode
  right?: React.ReactNode
}

export function Toolbar({ left, right }: IProps) {
  return (
    <div className={$toolbar.className}>
      <div className={$container.with({ left: true }).className}>
        {left}
      </div>
      <div className={$container.with({ left: false }).className}>
        {right}
      </div>
    </div>
  )
}
