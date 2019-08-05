import * as React from 'react'
import { Arhiv } from './arhiv'

export const ArhivContext = React.createContext<Arhiv>(null as any)

export function useArhiv(): Arhiv {
  const arhiv = React.useContext(ArhivContext)

  const [, setValue] = React.useState(arhiv.$updateTime.currentValue)

  React.useEffect(() => arhiv.$updateTime.subscribe(setValue), [])

  return arhiv
}
