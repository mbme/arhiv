import {
  isFunction,
  merge,
} from '~/utils'
import { createLogger } from '~/utils'
import { StylishRenderer } from './StylishRenderer'
import {
  IStyleObject,
  StylishDeclaration,
  StyleRuleResult,
  IProps,
  StyleTransformer,
} from './types'
import { applyTransformer } from './utils'

const log = createLogger('Stylish')

function mergeStyles(styles: StyleRuleResult[]): IStyleObject {
  const result: IStyleObject = {}

  for (const style of styles) {
    if (!style) {
      continue
    }

    for (const [key, value] of Object.entries(style)) {
      result[key] = merge(result[key], value)
    }
  }

  return result
}

// TODO:
// hashing without "avalanche"?
// atomic css?

export class StylishStyle {
  constructor(
    private _items: StylishDeclaration[],
    private _renderer: StylishRenderer,
    private _transformer?: StyleTransformer,
  ) { }

  with(props: IProps) {
    const items: IStyleObject[] = []

    // evaluate rules with props
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

    return new StylishStyle(
      items,
      this._renderer,
      this._transformer,
    )
  }

  and($style?: StylishStyle | IStyleObject) {
    if (!$style) {
      return this
    }

    return new StylishStyle(
      this._items.concat(...($style instanceof StylishStyle ? $style._items : [$style])),
      this._renderer,
      this._transformer,
    )
  }

  private _generateClassName(): string {
    let warned = true
    const styles = this._items.map((item) => {
      if (!isFunction(item)) {
        return item
      }

      if (!warned) {
        log.warn('Stylish has rules but no props were provided, empty object will be used instead')
        warned = true
      }

      return item({})
    })

    return this._renderer.render(applyTransformer(mergeStyles(styles), this._transformer))
  }

  private _className?: string
  get className() {
    this._className = this._className || this._generateClassName()

    return this._className
  }
}
