import * as React from 'react'
import {
  Obj,
} from '~/utils'
import {
  stylish,
} from './style'

interface IProps {
  as?: string
  children?: React.ReactNode
  [prop: string]: any
}
export class Box extends React.PureComponent<IProps> {
  static withProps(props: IProps) {
    return withProps(Box, props)
  }

  render() {
    const {
      as: Component = 'div',
      children,
      ...styleProps
    } = this.props

    const className = stylish(styleProps).className

    return React.createElement(Component, { className }, children)
  }
}

function withProps(Component: React.ComponentType, predefinedProps: Obj) {
  const ComponentWithProps = (props: Obj) => <Component {...predefinedProps} {...props} />
  ComponentWithProps.withProps = withProps.bind(null, ComponentWithProps)

  return ComponentWithProps
}

export function FlexRow({ justify = 'center', ...props }: Obj) {
  return (
    <Box
      display="flex"
      flexWrap="wrap"
      alignItems="center"
      justifyContent={justify as string}
      {...props}
    />
  )
}
