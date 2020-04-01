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

// eslint-disable-next-line max-len
interface IStylishProps<E extends keyof HTMLElementTagNameMap = 'div'> extends React.HTMLProps<HTMLElementTagNameMap[E]> {
  as?: E
  $style?: StyleArg
  innerRef?: React.RefObject<HTMLElementTagNameMap[E]>
  ref?: undefined
}

// eslint-disable-next-line max-len
export function StylishElement<E extends keyof HTMLElementTagNameMap = 'div'>({ $style, as, innerRef, ...props }: IStylishProps<E>) {
  const className = useStyles($style)

  return React.createElement(as || 'div', {
    ref: innerRef,
    className,
    ...props
  })
}
