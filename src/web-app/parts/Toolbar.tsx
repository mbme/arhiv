import * as React from 'react'
import { style } from 'typestyle'
import { theme } from '~/web-components'

const toolbarStyles = style({
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

const leftContainerStyles = style({
  $nest: {
    '& > *': {
      marginRight: '1rem',
    },
  },
})

const rightContainerStyles = style({
  $nest: {
    '& > *': {
      marginLeft: '1rem',
    },
  },
})

interface IProps {
  left?: React.ReactNode
  right?: React.ReactNode
}

export function Toolbar({ left, right }: IProps) {
  return (
    <div className={toolbarStyles}>
      <div className={leftContainerStyles}>
        {left}
      </div>
      <div className={rightContainerStyles}>
        {right}
      </div>
    </div>
  )
}
