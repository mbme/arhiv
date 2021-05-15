import * as React from 'react'
import { createContext } from '../utils'
import { WebRouter } from './web-router'

export const RouterContext = createContext<WebRouter>()

interface IProps {
  children?: React.ReactNode,
  hashBased?: boolean,
}

export function RouterProvider({ hashBased, children }: IProps) {
  const [router, setRouter] = React.useState<WebRouter | undefined>()

  React.useEffect(() => {
    const router = new WebRouter(hashBased)

    setRouter(router)

    return () => {
      router.stop()
    }
  }, [hashBased])

  return (
    <RouterContext.Provider value={router}>
      {router ? children : null}
    </RouterContext.Provider>
  )
}
