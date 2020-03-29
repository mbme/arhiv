import * as React from 'react'
import { StylishRenderer } from './StylishRenderer'
import {
  IStyleObject,
  StyleTransformer,
  StyleArg,
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

export function useStyles(...items: StyleArg[]) {
  const renderer = React.useContext(RendererContext)
  if (!renderer) {
    throw new Error("RendererContext isn't initialized yet")
  }

  const args = items.filter(item => item) as IStyleObject[]

  return React.useMemo(() => renderer.render(args), args)
}

export function useKeyframes(item: IStyleObject) {
  const renderer = React.useContext(RendererContext)
  if (!renderer) {
    throw new Error("RendererContext isn't initialized yet")
  }

  return React.useMemo(() => renderer.renderKeyframes(item), [item])
}

export function createStylishElement<E extends keyof HTMLElementTagNameMap>(element: E) {
  interface IStylishProps extends React.HTMLProps<HTMLElementTagNameMap[E]> {
    $styles: StyleArg[]
    innerRef?: React.RefObject<HTMLElementTagNameMap[E]>
    ref?: undefined
  }

  const StylishElement = ({ $styles, innerRef, ...props }: IStylishProps) => {
    const className = useStyles(...$styles)

    return React.createElement(element, {
      ref: innerRef,
      className,
      ...props
    })
  }

  return StylishElement
}
