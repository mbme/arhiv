import { Dict } from '@v/utils'
import { IStyleObject } from '@v/web-utils'
import { theme } from './theme'

const getThemeProp = (prop: keyof typeof theme) => (val: any) => (theme as any)[prop][val] || val
const getSpacing = getThemeProp('spacing')
const getColor = getThemeProp('color')
export const getAnimation = getThemeProp('animations')
const getFontSize = getThemeProp('fontSize')
const getFontFamily = getThemeProp('fontFamily')
const getZIndex = getThemeProp('zIndex')
const getBorder = getThemeProp('border')
const getBoxShadow = getThemeProp('boxShadow')

const mediaFrom = (minWidth: string) => `@media screen and (min-width: ${minWidth})`

type Rule = (val: any) => IStyleObject

const Rules: Dict<Rule> = {
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

  border: val => ({
    border: getBorder(val),
  }),
  borderTop: val => ({
    borderTop: getBorder(val),
  }),
  borderRight: val => ({
    borderRight: getBorder(val),
  }),
  borderBottom: val => ({
    borderBottom: getBorder(val),
  }),
  borderLeft: val => ({
    borderTop: getBorder(val),
  }),

  boxShadow: val => ({
    borderShadow: getBoxShadow(val),
  }),

  width: val => ({
    width: getSpacing(val),
  }),
  height: val => ({
    height: getSpacing(val),
  }),

  fontSize: val => ({
    fontSize: getFontSize(val),
  }),
  fontFamily: val => ({
    fontFamily: getFontFamily(val),
  }),
  zIndex: val => ({
    zIndex: getZIndex(val),
  }),


  color: val => ({
    color: getColor(val),
  }),
  backgroundColor: val => ({
    backgroundColor: getColor(val),
  }),
  bgColor: val => ({
    backgroundColor: getColor(val),
  }),

  relative: val => val && {
    position: 'relative',
  },

  absolute: val => val && {
    position: 'absolute',
  },

  bold: val => val && {
    fontWeight: 'bold',
  },

  uppercase: val => val && {
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

function mergeInto(target: IStyleObject, source: IStyleObject) {
  for (const [key, value] of Object.entries(source)) {
    target[key] = value // eslint-disable-line no-param-reassign
  }
}

export function stylishTransformer(src: IStyleObject): IStyleObject {
  const result: IStyleObject = {}

  for (const [prop, value] of Object.entries(src)) {
    const rule = Rules[prop]
    if (rule) {
      mergeInto(result, rule(value))
    } else {
      result[prop] = value
    }
  }

  return result
}
