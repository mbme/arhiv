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

const log = createLogger('Stylish')

interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}
type StyleRule = (props: IProps) => (IStyleObject | false | null | undefined)

interface IProps {
  $style?: Stylish
  [property: string]: any
}

function mergeStyles(styles: IStyleObject[]): IStyleObject {
  const result: IStyleObject = {}

  for (const style of styles) {
    for (const [key, value] of Object.entries(style)) {
      result[key] = value // TODO maybe merge value objects (queries, selectors)
    }
  }

  return result
}

const renderer = new Renderer(createStyleElement())
export class Stylish {
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

    const stylish = new Stylish(items)
    if (props.$style) {
      return stylish.and(props.$style)
    }

    return stylish
  }

  and($style?: Stylish) {
    if (!$style) {
      return this
    }

    return new Stylish(this._items.concat(...$style._items))
  }

  // FIXME cache classname
  get className() {
    if (this._hasRules) {
      log.warn('Stylish has rules but no props were provided, empty object will be used instead')
    }

    // FIXME ensure no rules
    const style = mergeStyles(this._items.map(item => isFunction(item) ? item({}) || {} : item))

    return renderer.render(style)
  }
}

// FIXME keyframes

export function stylish(...items: Array<IStyleObject | StyleRule>) {
  return new Stylish(items)
}
