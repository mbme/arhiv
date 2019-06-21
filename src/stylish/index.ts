import {
  createStyleElement,
  injectGlobalStyles,
} from './utils'
import {
  StylishDeclaration,
  IStyleObject,
  StyleRule,
} from './types'
import { Stylish } from './Stylish'
import { StylishKeyframes } from './StylishKeyframes'
import { Renderer } from './Renderer/Renderer'

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

export function keyframes(item: IStyleObject): string {
  return new StylishKeyframes(item, renderer).animationName
}
