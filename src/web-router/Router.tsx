import * as React from 'react'
import { WebRouter } from './web-router'

export const RouterContext = React.createContext<WebRouter>(null as any)

export function useRouter() {
  const router = React.useContext(RouterContext)
  if (!router) {
    throw new Error(`Can't useRouter: router not provided yet`)
  }

  return router
}
