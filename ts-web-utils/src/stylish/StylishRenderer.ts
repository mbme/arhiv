import { createLogger } from '@v/logger'
import {
  StyleNode,
  hash2className,
  hash2animation,
} from './StyleNode'
import {
  IStyleObject,
  StyleTransformer,
} from './types'
import {
  applyTransformer,
  mergeStyles,
} from './utils'

const log = createLogger('StylishRenderer')

export class StylishRenderer {
  private _rendered = new Set<string>()
  private _sheet: CSSStyleSheet

  constructor(
    el: HTMLStyleElement,
    private _transformer?: StyleTransformer,
  ) {
    if (!el.sheet) {
      throw new Error("Element doesn't have a stylesheet attached")
    }
    this._sheet = el.sheet as CSSStyleSheet
  }

  private _insert(rule: string) {
    this._sheet.insertRule(rule, this._sheet.cssRules.length)
  }

  render(...styles: IStyleObject[]): string {
    const style = new StyleNode(applyTransformer(mergeStyles(styles), this._transformer))
    const className = hash2className(style.hash)

    if (!this._rendered.has(className)) {
      for (const rule of style.intoCss()) {
        this._insert(rule)
      }

      this._rendered.add(className)
      this._logRendered()
    }

    return className
  }

  renderKeyframes(styleObj: IStyleObject): string {
    const style = new StyleNode(applyTransformer(styleObj, this._transformer))
    const animationName = hash2animation(style.hash)

    if (!this._rendered.has(animationName)) {
      this._insert(style.asKeyframes())

      this._rendered.add(animationName)
      this._logRendered()
    }

    return animationName
  }

  private _logRendered() {
    if (this._rendered.size % 100 === 0) {
      log.warn('rendered %s entities', this._rendered.size)
    }
  }
}
