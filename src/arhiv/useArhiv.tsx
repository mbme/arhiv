import * as React from 'react'
import { noop } from '~/utils'
import { Arhiv } from './arhiv'

export const ArhivContext = React.createContext<Arhiv>(null as any)

export function useArhiv(subscribe: boolean = true): Arhiv {
  const arhiv = React.useContext(ArhivContext)
  if (!arhiv) {
    throw new Error(`Can't useArhiv: arhiv not provided yet`)
  }

  const [, setValue] = React.useState(arhiv.$updateTime.currentValue)

  React.useEffect(() => {
    if (subscribe) {
      return arhiv.$updateTime.subscribe(setValue)
    }

    return noop
  }, [subscribe])

  return arhiv
}
