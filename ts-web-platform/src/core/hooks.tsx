import * as React from 'react'
import { isObject, isString } from '@v/utils'
import { IStyleObject } from './stylish'
import { RendererContext } from './StylishProvider'
import { StyleArg, IKeyframeProps } from './types'
import { stylishTransformer, getAnimation } from './stylish-transformer'
import { theme } from './theme'

function applyTransformer(style: IStyleObject): IStyleObject {
  const result = stylishTransformer(style)
  for (const [prop, value] of Object.entries(result)) {
    if (isObject(value)) {
      result[prop] = applyTransformer(value)
    }
  }

  return result
}

export function useStyles(...items: StyleArg[]) {
  const renderer = RendererContext.use()

  const args = items.map((item) => {
    if (item) {
      return applyTransformer(item as IStyleObject)
    }

    return item
  })

  return React.useMemo(() => renderer.render(args.filter(item => item) as IStyleObject[]), args)
}

export function useAnimation(item: IKeyframeProps | keyof typeof theme.animations) {
  const renderer = RendererContext.use()

  const arg = applyTransformer(isString(item) ? getAnimation(item) : item)

  return React.useMemo(() => renderer.renderKeyframes(arg), [arg])
}
