import * as React from 'react'
import {
  ReactiveValue,
} from './reactive-value'

export function useReactiveValue<T>(getValue: () => ReactiveValue<T>, deps: any[] = []) {
  const $value = React.useMemo(getValue, deps)

  const [value, setValue] = React.useState($value.currentValue)

  React.useEffect(() => $value.subscribe({ next: setValue }), [$value])

  return value
}
