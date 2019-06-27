import * as React from 'react'
import {
  stylish,
  Row,
  Spacer,
} from '~/web-platform'

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
    <Row
      position="sticky"
      top="0"
      bgColor="bg"
      height="60px"
      py="fine"
      mb="medium"
    >
      <div className={$container.with({ left: true }).className}>
        {left}
      </div>

      <Spacer />

      <div className={$container.with({ left: false }).className}>
        {right}
      </div>
    </Row>
  )
}
