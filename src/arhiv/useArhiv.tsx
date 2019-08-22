import * as React from 'react'
import { Arhiv } from './arhiv'

export const ArhivContext = React.createContext<Arhiv>(null as any)

export function useArhiv(): Arhiv {
  const arhiv = React.useContext(ArhivContext)
  if (!arhiv) {
    throw new Error(`Can't useArhiv: arhiv not provided yet`)
  }

  return arhiv
}
