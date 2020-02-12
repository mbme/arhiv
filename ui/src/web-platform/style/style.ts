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

export { injectGlobalStyles }

export type $Style = StylishStyle | IStyleObject

const renderer = new StylishRenderer(createStyleElement())

export function stylish(...items: StylishDeclaration[]) {
  return new StylishStyle(items, renderer, stylishTransformer)
}

export function keyframes(item: IStyleObject): string {
  return new StylishKeyframes(item, renderer, stylishTransformer).animationName
}
