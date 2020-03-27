import * as React from 'react'
import {
  Observable,
  promise$,
} from '@v/reactive'

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

export function usePromise<T>(
  getPromise: () => Promise<T>,
  deps: any[] = [],
): [T | undefined, boolean] {
  const [value, setValue] = React.useState<T | undefined>(undefined)
  const [error, setError] = React.useState<any>(undefined)

  React.useEffect(() => {
    const promise = getPromise()

    return promise$(promise).subscribe({
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

export function useBoolean(initialValue = false) {
  const [value, setValue] = React.useState<boolean>(initialValue)

  return {
    value,
    toggle() {
      setValue(!value)
    },
    set: setValue,
  }
}
