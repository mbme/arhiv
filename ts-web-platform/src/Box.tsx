import * as React from 'react'
import {
  isString,
  Procedure,
} from '@v/utils'
import {
  stylish,
  $Style,
} from './style'

export interface IProps {
  as?: React.ElementType
  children?: React.ReactNode
  $style?: $Style
  onClick?: Procedure
  innerRef?: React.RefObject<any>
  [prop: string]: any
}

export class Box extends React.PureComponent<IProps> {
  render() {
    const {
      as: Component = 'div',
      onClick,
      children,
      innerRef,
      $style,
      ...styleProps
    } = this.props

    const style = stylish(styleProps).and($style)

    if (isString(Component)) {
      return React.createElement(
        Component,
        {
          className: style.className,
          onClick,
          ref: innerRef,
        },
        children,
      )
    }

    return React.createElement(
      Component,
      {
        $style: style,
        onClick,
        ref: innerRef,
      },
      children,
    )
  }
}
