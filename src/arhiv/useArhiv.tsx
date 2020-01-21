import * as React from 'react'
import { ArhivReplica } from '~/arhiv/replica'

export const ArhivContext = React.createContext<ArhivReplica>(null as any)

export function useArhiv(): ArhivReplica {
  const arhiv = React.useContext(ArhivContext)
  if (!arhiv) {
    throw new Error("Can't useArhiv: arhiv not provided yet")
  }

  return arhiv
}
