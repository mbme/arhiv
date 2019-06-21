import {
  isObject,
} from '~/utils'
import {
  IStyleObject,
  StyleTransformer,
} from './types'

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

export function applyTransformer(style: IStyleObject, transformer?: StyleTransformer): IStyleObject {
  if (!transformer) {
    return style
  }

  const result = transformer(style)
  for (const [prop, value] of Object.entries(result)) {
    if (isObject(value)) {
      result[prop] = applyTransformer(value, transformer)
    }
  }

  return result
}
