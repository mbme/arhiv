import * as React from 'react'
import { ReactiveValue } from './reactive-value'

export function useReactiveValue<T>($value: ReactiveValue<T>) {
  const [value, setValue] = React.useState($value.currentValue)

  React.useEffect(() => $value.subscribe({ next: setValue }), [$value])

  return value
}

export function useReactiveValueMemo<T>(getValue: () => ReactiveValue<T>, deps?: any[]) {
  const $value = React.useMemo(getValue, deps)

  return useReactiveValue($value)
}
