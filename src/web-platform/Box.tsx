import * as React from 'react'
import { isString } from '~/utils'
import {
  stylish,
} from './style'

export interface IProps {
  as?: React.ElementType
  children?: React.ReactNode
  [prop: string]: any
}

export class Box extends React.PureComponent<IProps> {
  render() {
    const {
      as: Component = 'div',
      children,
      ...styleProps
    } = this.props

    if (isString(Component)) {
      const className = stylish(styleProps).className

      return React.createElement(Component, { className }, children)
    }

    return React.createElement(Component, { $style: styleProps }, children)
  }
}
