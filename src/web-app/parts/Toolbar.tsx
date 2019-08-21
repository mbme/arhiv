import * as React from 'react'
import {
  stylish,
  Row,
  theme,
} from '~/web-platform'

const $container = stylish({
  mx: `-${theme.spacing.fine}`, // compensate children margin

  '& > *': {
    mx: 'fine',
  },
})

interface IProps {
  children: React.ReactNode
}

export function Toolbar({ children }: IProps) {
  return (
    <Row
      position="sticky"
      top="0"
      bgColor="bg"
      height="60px"
      py="fine"
      mb="medium"
      $style={$container}
    >
      {children}
    </Row>
  )
}
