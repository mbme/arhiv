import {
  isFunction,
} from '~/utils'
import { createLogger } from '~/logger'
import { Renderer } from './Renderer'
const log = createLogger('Stylish')

interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}

type StyleRuleResult = IStyleObject | false | null | undefined
type StyleRule = (props: IProps) => StyleRuleResult

export type StylishDeclaration = IStyleObject | StyleRule

interface IProps {
  $style?: Stylish
  [property: string]: any
}

function mergeStyles(styles: StyleRuleResult[]): IStyleObject {
  const result: IStyleObject = {}

  for (const style of styles) {
    if (!style) {
      continue
    }

    for (const [key, value] of Object.entries(style)) {
      result[key] = value // TODO maybe merge value objects (queries, selectors)
    }
  }

  return result
}

// TODO:
// keyframes
// custom @media support?
// do not sort props before hashing
// merge objects
// hashing without "avalanche"?
// atomic css?

export class Stylish {
  constructor(
    private _items: StylishDeclaration[],
    private _renderer: Renderer,
  ) { }

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

    const stylish = new Stylish(items, this._renderer)
    if (props.$style) {
      return stylish.and(props.$style)
    }

    return stylish
  }

  and($style?: Stylish) {
    if (!$style) {
      return this
    }

    return new Stylish(this._items.concat(...$style._items), this._renderer)
  }

  private _generateClassName(): string {
    let warned = true
    const style = mergeStyles(this._items.map((item) => {
      if (!isFunction(item)) {
        return item
      }

      if (!warned) {
        log.warn('Stylish has rules but no props were provided, empty object will be used instead')
        warned = true
      }

      return item({})
    }))

    return this._renderer.render(style)
  }

  private _className?: string
  get className() {
    this._className = this._className || this._generateClassName()

    return this._className
  }
}
