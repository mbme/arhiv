import * as React from 'react'
import { Counter } from '~/utils'
import { IsodbWebClient } from './isodb-web-client'

export const IsodbContext = React.createContext<IsodbWebClient>(null as any)

const counter = new Counter()

export function useIsodb(): IsodbWebClient {
  const isodb = React.useContext(IsodbContext)

  const [, setValue] = React.useState(counter._value)

  React.useEffect(() => {
    const onUpdate = () => setValue(counter.incAndGet())

    isodb.events.on('db-update', onUpdate)
    isodb.events.on('isodb-lock', onUpdate)

    onUpdate() // values might have changed between render() and componentDidMount()

    return () => {
      isodb.events.off('db-update', onUpdate)
      isodb.events.off('isodb-lock', onUpdate)
    }
  }, [])

  return isodb
}
