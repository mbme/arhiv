import * as React from 'react'
import { IsodbWebClient } from './isodb-web-client'

export const IsodbContext = React.createContext<IsodbWebClient>(null as any)

export function useIsodb(): IsodbWebClient {
  const isodb = React.useContext(IsodbContext)

  const [, setValue] = React.useState(0)

  React.useEffect(() => {
    const onUpdate = () => setValue(Date.now())
    isodb.events.on('db-update', onUpdate)
    isodb.events.on('isodb-lock', onUpdate)

    return () => {
      isodb.events.off('db-update', onUpdate)
      isodb.events.off('isodb-lock', onUpdate)
    }
  }, [])

  return isodb
}
