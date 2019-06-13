import { Obj } from '~/utils'
import { Renderer } from './Renderer/Renderer'
import {
  createStyleElement,
  injectGlobalStyles,
} from './utils'
import {
  Stylish,
  StylishDeclaration,
  IStyleObject,
  StyleRule,
} from './Stylish'

export {
  injectGlobalStyles,
  Stylish,
  IStyleObject,
  StyleRule,
  StylishDeclaration,
}

const renderer = new Renderer(createStyleElement())

export function stylish(...items: StylishDeclaration[]) {
  return new Stylish(items, renderer)
}

export function keyframes(item: Obj) {
  return renderer.renderKeyframe(item)
}
