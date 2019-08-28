import * as React from 'react'
import { ReactiveValue } from '../reactive-value'

export function useReactiveValue<T>($value: ReactiveValue<T>) {
  const [value, setValue] = React.useState($value.currentValue)

  React.useEffect(() => $value.subscribe(setValue), [$value])

  return value
}
