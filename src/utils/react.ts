import * as React from 'react'
import {
  ReactiveValue,
} from './reactive-value'
import { isEqualArray } from './array'

function useGetter<T>(getValue: () => T, deps: any[] = []): T {
  const valueRef = React.useRef<T | undefined>(undefined)
  const depsRef = React.useRef<any[] | undefined>(undefined)

  if (!depsRef.current) {
    depsRef.current = deps
  }

  if (!valueRef.current || !isEqualArray(deps, depsRef.current)) {
    valueRef.current = getValue()
  }

  return valueRef.current
}

export function useReactiveValue<T>(getValue: () => ReactiveValue<T>, deps: any[] = [], completeOnUnsub = false) {
  const $value = useGetter(getValue, deps)

  const [value, setValue] = React.useState($value.currentValue)

  React.useEffect(() => {
    const unsub = $value.subscribe({ next: setValue })

    return () => {
      console.error('UNSUB');
      if (completeOnUnsub) {
        $value.complete()
      } else {
        unsub()
      }
    }
  }, [$value])

  return value
}
