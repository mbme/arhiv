import * as React from 'react'
import { useObservable } from '@v/web-utils'
import { noop } from '@v/utils'
import { FocusManagerContext } from './FocusContext'

// FIXME
// onEnter
// onFocus/onBlur
export function useFocusable<T extends HTMLElement>(): [boolean, React.Ref<T>] {
  const [ref, setRef] = React.useState<T | null>(null)

  const isNodeFocused = (node: HTMLElement | null) => !!node && node === ref

  const context = FocusManagerContext.use()

  const [isFocused] = useObservable(
    () => context.selected$.value$.map(isNodeFocused),
    [context.selected$, ref],
    isNodeFocused(context.selected$.value),
  )

  React.useEffect(() => {
    if (!ref) {
      return noop
    }

    return context.registerNode(ref)
  }, [ref])

  return [isFocused, setRef]
}
