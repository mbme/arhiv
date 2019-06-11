import {
  isFunction,
} from '~/utils'
import { createLogger } from '~/logger'
import { Renderer } from './Renderer/Renderer'
import {
  createStyleElement,
  injectGlobalStyles,
} from './utils'

export { injectGlobalStyles }

const log = createLogger('Styler')

interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}
interface IProps {
  $style?: Style
  [property: string]: any
}
type StyleRule = (props: IProps) => (IStyleObject | false | null | undefined)

export type Style = Styler | IStyleObject

// FIXME merge properly
function mergeStyles(styles: IStyleObject[]): IStyleObject {
  return styles.reduce((acc, style) => ({ ...acc, ...style }), {})
}

const renderer = new Renderer(createStyleElement())
class Styler {
  private _hasRules: boolean

  constructor(private _items: Array<IStyleObject | StyleRule>) {
    this._hasRules = !!this._items.find(isFunction)
  }

  with(props: IProps) {
    const items: IStyleObject[] = []

    for (const item of this._items) {
      if (isFunction(item)) {
        const result = item(props)
        if (result) {
          items.push(result)
        }
        continue
      }

      items.push(item)
    }

    const styler = new Styler(items)
    if (props.$style) {
      return styler.and(props.$style)
    }

    return styler
  }

  and($style?: Style) { // FIXME improve this to be incompatible with and() method
    if (!$style) {
      return this
    }

    const items = $style instanceof Styler ? $style._items : [$style]

    return new Styler(this._items.concat(...items))
  }

  // FIXME cache classname
  get className() {
    if (this._hasRules) {
      log.warn('Styler has rules but no props were provided, empty object will be used instead')
    }

    // FIXME ensure no rules
    const style = mergeStyles(this._items.map(item => isFunction(item) ? item({}) || {} : item))

    return renderer.render(style)
  }
}

// TODO keyframes

export function styleRules(...items: Array<IStyleObject | StyleRule>) {
  return new Styler(items)
  // const preps: Array<string | StyleRule> = items.map((item) => {
  //   if (isObject(item)) {
  //     return renderer.render(item)
  //   }

  //   return item
  // })

  // return (props: Obj = {}) => preps.map((item) => {
  //   // TODO handle props.$style

  //   if (isString(item)) {
  //     return item
  //   }

  //   const ruleResult = item(props)
  //   if (!ruleResult) {
  //     return ''
  //   }

  //   return renderer.render(ruleResult)
  // }).join(' ')
}
