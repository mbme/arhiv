import * as React from 'react'
import { isString } from '~/utils'
import {
  stylish,
  $Style,
} from './style'

export interface IProps {
  as?: React.ElementType
  children?: React.ReactNode
  $style?: $Style
  [prop: string]: any
}

export class Box extends React.PureComponent<IProps> {
  render() {
    const {
      as: Component = 'div',
      children,
      $style,
      ...styleProps
    } = this.props

    const style = stylish(styleProps).and($style)

    if (isString(Component)) {
      return React.createElement(Component, { className: style.className }, children)
    }

    return React.createElement(Component, { $style: style }, children)
  }
}
