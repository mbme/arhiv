import * as React from 'react'
import { createContext } from '../context'
import { HotkeysResolver } from './resolver'

export const HotkeysResolverContext = createContext<HotkeysResolver>()

interface IProps {
  children: React.ReactNode
}

export function HotkeysResolverProvider({ children }: IProps) {
  const [resolver] = React.useState(() => new HotkeysResolver())

  React.useEffect(() => {
    return resolver.registerDocument(document)
  }, [])

  return (
    <HotkeysResolverContext.Provider value={resolver}>
      {children}
    </HotkeysResolverContext.Provider>
  )
}
