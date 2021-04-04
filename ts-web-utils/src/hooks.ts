import * as React from 'react'
import {
  Cell,
  Observable,
  promise$,
} from '@v/reactive'
import { Counter, noop } from '@v/utils'

export function useObservable<T, K = undefined>(
  getObservable$: () => Observable<T>,
  deps: any[] = [],
  initialValue: K,
): [T | K, any] {
  const [value, setValue] = React.useState<T | K>(initialValue)
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

export function useCell<T>(cell$: Cell<T>): [T, any] {
  return useObservable(() => cell$.value$, [cell$], cell$.value)
}

export function usePromise<T>(
  getPromise: () => Promise<T>,
  deps: any[] = [],
): [T | undefined, any] {
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

const _counter = new Counter()
export function useCounter() {
  const [counter] = React.useState<number>(() => _counter.incAndGet())

  return counter
}

export function useDebouncedCallback(
  cb: (...args: any[]) => void,
  timeoutMs: number,
  deps: any[] = [],
) {
  const cbRef = React.useRef(cb)
  const timeoutRef = React.useRef<number | undefined>(undefined)

  // keep callback reference up-to-date
  React.useEffect(() => {
    cbRef.current = cb
  }, deps)

  // clear pending timeout on unmount
  React.useEffect(() => {
    return () => {
      window.clearTimeout(timeoutRef.current)
    }
  }, [])

  return React.useCallback((...args: any[]) => {
    window.clearTimeout(timeoutRef.current)

    timeoutRef.current = window.setTimeout(() => {
      cbRef.current(...args)
    }, timeoutMs)
  }, [])
}


export function useDebounced<T>(value: T, timeoutMs: number, isValid = true): T {
  const [debouncedValue, setDebouncedValue] = React.useState<T>(value)

  React.useEffect(() => {
    if (!isValid) {
      return noop
    }

    const timeout = window.setTimeout(() => setDebouncedValue(value), timeoutMs)

    return () => window.clearTimeout(timeout)
  }, [value, isValid])

  return debouncedValue
}

export function useIsWindowFocused(): boolean {
  const [isFocused, setIsFocused] = React.useState(() => document.hasFocus())

  React.useEffect(() => {
    const updateFocus = () => {
      setIsFocused(document.hasFocus())
    }

    window.addEventListener('focus', updateFocus, { passive: true })
    window.addEventListener('blur', updateFocus, { passive: true })

    return () => {
      window.removeEventListener('focus', updateFocus)
      window.removeEventListener('blur', updateFocus)
    }
  }, [])

  return isFocused
}

export function useOnBeforeUnload() {
  React.useEffect(() => {
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      e.preventDefault()

      // Chrome requires returnValue to be set
      e.returnValue = ''
    }

    window.addEventListener('beforeunload', onBeforeUnload)

    return () => {
      window.removeEventListener('beforeunload', onBeforeUnload)
    }
  }, [])
}
