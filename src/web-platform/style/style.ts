import {
  StylishRenderer,
  createStyleElement,
  injectGlobalStyles,
  StylishDeclaration,
  StylishStyle,
  StylishKeyframes,
  IStyleObject,
} from '~/stylish'
import { stylishTransformer } from './stylish-transformer'

export {
  injectGlobalStyles,
  StylishStyle,
}

// FIXME
// stylish should handle shortcuts, mediaqueries and theme
// $style attr should accept objects
// Box should accept attributes without $ prefix
// handle media queries

const renderer = new StylishRenderer(createStyleElement())

export function stylish(...items: StylishDeclaration[]) {
  return new StylishStyle(items, renderer, stylishTransformer)
}

export function keyframes(item: IStyleObject): string {
  return new StylishKeyframes(item, renderer, stylishTransformer).animationName
}
