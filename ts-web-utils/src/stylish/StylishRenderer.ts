import { createLogger } from '@v/logger'
import {
  StyleNode,
  hash2className,
  hash2animation,
} from './StyleNode'
import { IStyleObject } from './types'
import { mergeStyles } from './utils'

const log = createLogger('StylishRenderer')

export class StylishRenderer {
  private _rendered = new Set<string>()
  private _sheet: CSSStyleSheet

  constructor(el: HTMLStyleElement) {
    if (!el.sheet) {
      throw new Error("Element doesn't have a stylesheet attached")
    }
    this._sheet = el.sheet
  }

  private _insert(rule: string) {
    this._sheet.insertRule(rule, this._sheet.cssRules.length)
  }

  render(styles: IStyleObject[]): string {
    const style = new StyleNode(mergeStyles(styles))
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
    const style = new StyleNode(styleObj)
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
