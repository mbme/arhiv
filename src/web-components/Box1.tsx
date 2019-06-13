import * as React from 'react'
import { Obj } from '~/utils'
import {
  stylish,
} from '~/stylish'
import theme from './theme'

// tslint:disable-next-line:no-unsafe-any
const getThemeProp = (prop: string) => (val: any) => (theme as Obj)[prop][val] || val
const getSpacing = getThemeProp('spacing')

type Rule = (val: any) => Obj
const Rules: { [name: string]: Rule | undefined } = {
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
    fontSize: (theme.fontSize as any)[val] || val,
  }),
  fontFamily: val => ({
    fontFamily: (theme.fontFamily as any)[val] || val,
  }),
  zIndex: val => ({
    zIndex: (theme.zIndex as any)[val],
  }),
}

function mergeInto(target: Obj, source: Obj) {
  for (const [key, value] of Object.entries(source)) {
    target[key] = value
  }
}

function splitProps(allProps: Obj) {
  let as: any = 'div'
  const props: Obj = {}
  const $props: Obj = {}

  for (const [key, value] of Object.entries(allProps)) {
    if (key === 'as') {
      as = value
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
      mergeInto($props, $rule(value))
    } else {
      $props[prop] = value
    }
  }

  props.className = stylish($props)

  return { as, props }
}

export class Box extends React.PureComponent<Obj> {
  render() {
    const {
      as: Component,
      props,
    } = splitProps(this.props)

    return (
      // tslint:disable-next-line:no-unsafe-any
      <Component {...props} />
    )
  }
}
