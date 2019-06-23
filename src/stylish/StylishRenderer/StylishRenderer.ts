import { Obj } from '~/utils'
import {
  StyleNode,
  hash2className,
  hash2animation,
} from './StyleNode'

export class StylishRenderer {
  private _rendered = new Set<string>()

  constructor(private _el: HTMLStyleElement) { }

  private _insert(rule: string) {
    (this._el.sheet as CSSStyleSheet).insertRule(rule)
  }

  render(styleObj: Obj): string {
    const style = new StyleNode(styleObj)
    const className = hash2className(style.hash)

    if (!this._rendered.has(className)) {
      for (const rule of style.intoCss()) {
        this._insert(rule)
      }

      this._rendered.add(className)
    }

    return className
  }

  renderKeyframes(styleObj: Obj): string {
    const style = new StyleNode(styleObj)
    const animationName = hash2animation(style.hash)

    if (!this._rendered.has(animationName)) {
      this._insert(style.asKeyframes())
      this._rendered.add(animationName)
    }

    return animationName
  }
}
