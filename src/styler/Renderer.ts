import { Obj } from '~/utils'
import { Style } from './Style'
import { hash2className } from './utils'

export class Renderer {
  private _classes = new Set<string>()

  constructor(private _el: HTMLStyleElement) { }

  render(styleObj: Obj): string {
    const style = new Style(styleObj)
    const className = hash2className(style.hash)

    if (!this._classes.has(className)) {
      for (const rule of style.intoCss()) {
        (this._el.sheet as CSSStyleSheet).insertRule(rule)
      }

      this._classes.add(className)
    }

    return className
  }
}
