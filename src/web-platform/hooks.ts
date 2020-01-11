import * as React from 'react'
import {
  noop,
} from '~/utils'
import {
  Observable,
  promise$,
} from '~/reactive'

export function useObservable<T>(
  getObservable$: () => Observable<T>,
  deps: any[] = [],
): [T | undefined, any] {
  const [value, setValue] = React.useState<T | undefined>(undefined)
  const [error, setError] = React.useState<any>(undefined)

  React.useEffect(() => {
    const o$ = getObservable$()

    return o$.subscribe({
      next(newValue) {
        setValue(newValue)
      },

      error(e) {
        setError(e)
      },
    })
  }, deps)

  return [value, error]
}

// FIXME remove this
export function useObservableOld<T>(
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
        throw new Error(`Got an error from observable: ${e}`)
      },
    })
  }, [observable$])

  return [value, isReady]
}

export function usePromise<T>(getPromise: () => Promise<T>, deps: any[] = []): [T | undefined, boolean] {
  const [promise, setPromise] = React.useState<Promise<T> | undefined>(undefined)
  const [value, setValue] = React.useState<T | undefined>(undefined)
  const [isReady, setIsReady] = React.useState<boolean>(false)

  React.useEffect(() => {
    setPromise(getPromise())
    setIsReady(false)
  }, deps)

  React.useEffect(() => {
    if (!promise) {
      return noop
    }

    return promise$(promise).subscribe({
      next(newValue) {
        setValue(newValue)
        setIsReady(true)
      },

      error(e) {
        throw new Error(`Got an error from promise: ${e}`)
      },
    })
  }, [promise])

  return [value, isReady]
}
