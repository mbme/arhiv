import {
  isObject,
  isString,
  Obj,
} from '~/utils'
import { Renderer } from './Renderer'
import { createStyleElement } from './utils'

type Rule = (props: Obj) => (Obj | false | null | undefined)

const renderer = new Renderer(createStyleElement())

// TODO keyframes

export function style(...items: Obj[]): string {
  const styleClass = items.map(item => renderer.render(item)).join(' ')

  // add whitespace just to simplify class concatenation: style({ width: 100 }) + 'other-class'
  return styleClass + ' '
}

export function styleRules(...items: Array<Rule | Obj>): (props: Obj, className?: string) => string {
  const preps: Array<string | Rule> = items.map((item) => {
    if (isObject(item)) {
      return renderer.render(item)
    }

    return item
  })

  return (props: Obj, className?: string) => {
    const c1 = preps.map((item) => {
      if (isString(item)) {
        return item
      }

      const ruleResult = item(props)
      if (!ruleResult) {
        return ''
      }

      return renderer.render(ruleResult)
    }).join(' ')

    return `${c1} ${className}`
  }
}

export function injectGlobalStyles(styles: string) {
  const el = createStyleElement(true)
  el.textContent = styles
}
