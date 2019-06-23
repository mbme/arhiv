import * as React from 'react'
import {
  Obj,
  isFunction,
} from '~/utils'
import { stylish } from './style'
import theme from './theme'

// tslint:disable-next-line:no-unsafe-any
const getThemeProp = (prop: string) => (val: any) => (theme as Obj)[prop][val] || val
const getSpacing = getThemeProp('spacing')

type Rule = (val: any) => Obj
const Rules: { [name: string]: Rule | Obj | undefined } = {
  m: val => ({
    margin: getSpacing(val),
  }),
  mx: val => ({
    marginLeft: getSpacing(val),
    marginRight: getSpacing(val),
  }),
  my: val => ({
    marginTop: getSpacing(val),
    marginBottom: getSpacing(val),
  }),
  mt: val => ({
    marginTop: getSpacing(val),
  }),
  mr: val => ({
    marginRight: getSpacing(val),
  }),
  mb: val => ({
    marginBottom: getSpacing(val),
  }),
  ml: val => ({
    marginLeft: getSpacing(val),
  }),

  p: val => ({
    padding: getSpacing(val),
  }),
  px: val => ({
    paddingLeft: getSpacing(val),
    paddingRight: getSpacing(val),
  }),
  py: val => ({
    paddingTop: getSpacing(val),
    paddingBottom: getSpacing(val),
  }),
  pt: val => ({
    paddingTop: getSpacing(val),
  }),
  pr: val => ({
    paddingRight: getSpacing(val),
  }),
  pb: val => ({
    paddingBottom: getSpacing(val),
  }),
  pl: val => ({
    paddingLeft: getSpacing(val),
  }),

  top: val => ({
    top: getSpacing(val),
  }),
  left: val => ({
    left: getSpacing(val),
  }),
  bottom: val => ({
    bottom: getSpacing(val),
  }),
  right: val => ({
    right: getSpacing(val),
  }),

  fontSize: val => ({
    fontSize: getThemeProp('fontSize')(val),
  }),
  fontFamily: val => ({
    fontFamily: getThemeProp('fontFamily')(val),
  }),
  zIndex: val => ({
    zIndex: getThemeProp('zIndex')(val),
  }),

  relative: {
    position: 'relative',
  },
}

function mergeInto(target: Obj, source: Obj) {
  for (const [key, value] of Object.entries(source)) {
    target[key] = value
  }
}

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

      // TODO handle media queries
      const prop = key.substring(1)

      const $rule = Rules[prop]
      if ($rule) {
        mergeInto($props, isFunction($rule) ? $rule(value) : $rule)
      } else {
        $props[prop] = value
      }
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
