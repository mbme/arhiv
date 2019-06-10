import {
  isObject,
  isString,
  Obj,
} from '~/utils'
import { Renderer } from './Renderer/Renderer'
import {
  createStyleElement,
  injectGlobalStyles,
} from './utils'
import {
  Style,
  StyleRule,
} from './types'

export {
  injectGlobalStyles,
  Style,
}

const renderer = new Renderer(createStyleElement())

// TODO keyframes

export function styleRules(...items: Style[]) {
  const preps: Array<string | StyleRule> = items.map((item) => {
    if (isObject(item)) {
      return renderer.render(item)
    }

    return item
  })

  return (props: Obj = {}) => preps.map((item) => {
    // TODO handle props.$style

    if (isString(item)) {
      return item
    }

    const ruleResult = item(props)
    if (!ruleResult) {
      return ''
    }

    return renderer.render(ruleResult)
  }).join(' ')
}
