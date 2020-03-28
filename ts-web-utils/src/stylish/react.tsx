import * as React from 'react'
import { StylishRenderer } from './StylishRenderer'
import {
  IStyleObject,
  StyleTransformer,
} from './types'
import {
  createStyleElement,
} from './utils'

const RendererContext = React.createContext<StylishRenderer | undefined>(undefined)

interface IProps {
  children?: React.ReactNode,
  transformer?: StyleTransformer,
}

export function StylishProvider({ transformer, children }: IProps) {
  const [renderer, setRenderer] = React.useState<StylishRenderer | undefined>()

  React.useEffect(() => {
    const el = createStyleElement()

    setRenderer(new StylishRenderer(el, transformer))

    return () => {
      el.remove()
    }
  }, [transformer])

  return (
    <RendererContext.Provider value={renderer}>
      {children}
    </RendererContext.Provider>
  )
}

export function useStyles(...items: Array<IStyleObject | undefined | null | false | ''>) {
  const renderer = React.useContext(RendererContext)
  if (!renderer) {
    throw new Error("RendererContext isn't initialized yet")
  }

  const args = items.filter(item => item) as IStyleObject[]

  return React.useMemo(() => renderer.render(...args), args)
}

export function useKeyframes(item: IStyleObject) {
  const renderer = React.useContext(RendererContext)
  if (!renderer) {
    throw new Error("RendererContext isn't initialized yet")
  }

  return React.useMemo(() => renderer.renderKeyframes(item), [item])
}
