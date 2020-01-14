import {
  Obj,
  isFunction,
} from '~/utils'
import { theme } from './theme'

// tslint:disable-next-line:no-unsafe-any
const getThemeProp = (prop: keyof typeof theme) => (val: any) => (theme as Obj)[prop][val] || val
const getSpacing = getThemeProp('spacing')
const getColor = getThemeProp('color')

const mediaFrom = (minWidth: string) => `@media screen and (min-width: ${minWidth})`

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

  width: val => ({
    width: getSpacing(val),
  }),
  height: val => ({
    height: getSpacing(val),
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

  color: val => ({
    color: getColor(val),
  }),
  bgColor: val => ({
    backgroundColor: getColor(val),
  }),

  relative: {
    position: 'relative',
  },

  bold: {
    fontWeight: 'bold',
  },

  uppercase: {
    textTransform: 'uppercase',
  },

  hidden: val => ({
    display: val ? 'none' : undefined,
  }),

  fromSm: val => ({
    [mediaFrom(theme.breakpoints.sm)]: val,
  }),
  fromMd: val => ({
    [mediaFrom(theme.breakpoints.md)]: val,
  }),
  fromLg: val => ({
    [mediaFrom(theme.breakpoints.lg)]: val,
  }),
}

function mergeInto(target: Obj, source: Obj) {
  for (const [key, value] of Object.entries(source)) {
    target[key] = value
  }
}

export function stylishTransformer(src: Obj): Obj {
  const result: Obj = {}

  for (const [prop, value] of Object.entries(src)) {
    const rule = Rules[prop]
    if (rule) {
      mergeInto(result, isFunction(rule) ? rule(value) : rule)
    } else {
      result[prop] = value
    }
  }

  return result
}
