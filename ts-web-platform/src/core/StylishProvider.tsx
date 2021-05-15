import * as React from 'react'
import { createContext } from '../utils'
import {
  createStyleElement,
  StylishRenderer,
} from './stylish'

export const RendererContext = createContext<StylishRenderer>()

interface IProps {
  children?: React.ReactNode,
}

export function StylishProvider({ children }: IProps) {
  const [renderer, setRenderer] = React.useState<StylishRenderer | undefined>()

  React.useEffect(() => {
    const el = createStyleElement()

    setRenderer(new StylishRenderer(el))

    return () => {
      el.remove()
    }
  }, [])

  return (
    <RendererContext.Provider value={renderer}>
      {renderer ? children : null}
    </RendererContext.Provider>
  )
}
