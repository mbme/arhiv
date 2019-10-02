import * as React from 'react'
import { noop } from './misc'
import { createLogger } from './logger'
import { Observable } from './reactive'

const log = createLogger('react-utils')

export function useObservable<T>(getObservable$: () => Observable<T>, deps: any[] = []) {
  const [observable$, setObservable] = React.useState<Observable<T> | undefined>(undefined)
  const [value, setValue] = React.useState<T | undefined>(undefined)

  React.useEffect(() => {
    setObservable(getObservable$())
  }, deps)

  React.useEffect(() => {
    if (!observable$) {
      return noop
    }

    return observable$.subscribe({
      next: setValue,
      error(e) {
        log.warn('Got and error from observable', e)
      },
    })
  }, [observable$])

  return value
}
