import * as React from 'react'
import { useCell } from '@v/web-utils'
import { noop } from '@v/utils'
import { FocusManagerContext } from './FocusContext'

// onEnter
// onFocus/onBlur
export function useFocusable<T extends HTMLElement>(): [boolean, React.Ref<T>] {
  const [ref, setRef] = React.useState<T | null>(null)
  const context = FocusManagerContext.use()
  const [focusedRef] = useCell(context.selected$)

  const isFocused = !!focusedRef && focusedRef === ref

  React.useEffect(() => {
    if (!ref) {
      return noop
    }

    return context.registerNode(ref)
  }, [ref])

  return [isFocused, setRef]
}
