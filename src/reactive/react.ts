import * as React from 'react'
import {
  noop,
  createLogger,
} from '~/utils'
import { Observable } from './observable'

const log = createLogger('react-utils')

export function useObservable<T>(
  getObservable$: () => Observable<T>,
  deps: any[] = [],
  timeoutMs: number = 3000,
): [T | undefined, boolean] {
  const [observable$, setObservable] = React.useState<Observable<T> | undefined>(undefined)
  const [value, setValue] = React.useState<T | undefined>(undefined)
  const [isReady, setIsReady] = React.useState<boolean>(false)

  React.useEffect(() => {
    setObservable(getObservable$())

    setIsReady(false)
    const timeoutId = setTimeout(() => {
      setIsReady(true)
    }, timeoutMs)

    return () => clearTimeout(timeoutId)
  }, deps)

  React.useEffect(() => {
    if (!observable$) {
      return noop
    }

    return observable$.subscribe({
      next(newValue) {
        setValue(newValue)
        setIsReady(true)
      },
      error(e) {
        log.warn('Got and error from observable', e)
      },
    })
  }, [observable$])

  return [value, isReady]
}
