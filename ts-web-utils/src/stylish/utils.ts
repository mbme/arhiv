import { merge } from '@v/utils'
import { IStyleObject } from './types'

export function createStyleElement(prepend = false) {
  const el = document.createElement('style')
  el.setAttribute('type', 'text/css')

  if (prepend) {
    document.head.prepend(el)
  } else {
    document.head.appendChild(el)
  }

  return el
}

export function injectGlobalStyles(styles: string) {
  const el = createStyleElement(true)
  el.textContent = styles
}

export function mergeStyles(styles: IStyleObject[]): IStyleObject {
  const result: IStyleObject = {}

  for (const style of styles) {
    for (const [key, value] of Object.entries(style)) {
      result[key] = merge(result[key], value)
    }
  }

  return result
}
