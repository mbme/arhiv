import {
  Obj,
  isObject,
  hashCode,
  camelCase2kebabCase,
} from '~/utils'
import { hash2class } from './utils'

// Helper class, which parses & transforms style object
export class Style {
  private media: Map<string, Style> = new Map()
  private nested: Map<string, Style> = new Map()
  private propsStr: string

  hash: string

  constructor(styleObj: Obj, isTopLevel = true, isInMediaQuery = false) {
    const props = []

    for (const [prop, value] of Object.entries(styleObj)) {
      if (!isObject(value)) {
        props.push(`${camelCase2kebabCase(prop)}: ${value}`)
        continue
      }

      // nested media rule, e.g. media query
      // https://developer.mozilla.org/en-US/docs/Web/CSS/At-rule
      if (prop.startsWith('@')) {
        if (!isTopLevel) {
          throw new Error('media queries are allowed only on the top level')
        }

        this.media.set(prop, new Style(value, false, true))
      } else {
        if (!isTopLevel && !isInMediaQuery) {
          throw new Error('nested blocks are allowed only on the top level or in media queries')
        }

        // nested selector, like "&:hover" or ".test & .other" etc
        this.nested.set(prop, new Style(value, false, false))
      }
    }

    this.propsStr = props.sort().join(';')
    this.hash = this._calculateHash()
  }

  private _calculateHash(): string {
    const hashData = [
      hashCode(this.propsStr).toString(),
    ]

    for (const [prop, style] of this.media.entries()) {
      hashData.push(`${prop}-${style.hash}`)
    }

    for (const [prop, style] of this.nested.entries()) {
      hashData.push(`${prop}-${style.hash}`)
    }

    return hashCode(hashData.join('-')).toString()
  }

  private _serializeProps(selector: string): string {
    return `${selector} { ${this.propsStr} }`
  }

  private _serializeMedia(cssClass: string): string[] {
    const styles = []

    for (const [media, style] of this.media.entries()) {
      const mediaStyles = [
        style._serializeProps(cssClass),
        ...style._serializeNested(cssClass),
      ].join('\n')

      styles.push(`${media} { ${mediaStyles} }`)
    }

    return styles
  }

  private _serializeNested(cssClass: string): string[] {
    const styles = []

    for (const [selectorTemplate, style] of this.nested.entries()) {
      const selector = selectorTemplate.replace('&', cssClass)
      styles.push(style._serializeProps(selector))
    }

    return styles
  }

  // Convert style object into array of css rules
  intoCss(): string[] {
    const cssClass = hash2class(this.hash)

    return [
      this._serializeProps(cssClass),
      ...this._serializeNested(cssClass),
      ...this._serializeMedia(cssClass),
    ]
  }
}
