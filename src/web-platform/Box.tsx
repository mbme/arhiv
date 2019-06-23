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

  splitProps() {
    const props: Obj = {}
    const $props: Obj = {}

    for (const [key, value] of Object.entries(this.props)) {
      if (key === 'as') { // skip "as" prop cause we handle it separately
        continue
      }

      if (!key.startsWith('$')) {
        props[key] = value
        continue
      }

      $props[key.substring(1)] = value
    }

    props.className = stylish($props)

    return props
  }

  render() {
    const {
      as: Component = 'as',
    } = this.props

    const props = this.splitProps()

    return (
      <Component {...props} />
    )
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
      $display="flex"
      $flexWrap="wrap"
      $alignItems="center"
      $justifyContent={justify as string}
      {...props}
    />
  )
}
