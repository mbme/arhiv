import {
  StylishRenderer,
  createStyleElement,
  injectGlobalStyles,
  StylishDeclaration,
  StylishStyle,
  StylishKeyframes,
  IStyleObject,
} from '~/stylish'

export {
  injectGlobalStyles,
  StylishStyle,
}

// FIXME
// * stylish should handle shortcuts, mediaqueries and theme
// $style attr should accept objects
// Box should accept attributes without $ prefix

const renderer = new StylishRenderer(createStyleElement())

export function stylish(...items: StylishDeclaration[]) {
  return new StylishStyle(items, renderer)
}

export function keyframes(item: IStyleObject): string {
  return new StylishKeyframes(item, renderer).animationName
}
