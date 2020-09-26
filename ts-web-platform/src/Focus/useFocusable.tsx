import * as React from 'react'
import { useObservable } from '@v/web-utils'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'

// FIXME
// onEnter
// onFocus/onBlur
export function useFocusable<T extends HTMLElement>(): [boolean, React.Ref<T>] {
  const [ref, setRef] = React.useState<T | null>(null)

  const context = React.useContext(FocusManagerContext)
  if (!context) {
    throw new Error('FocusManager must be provided')
  }

  const [isFocused] = useObservable(
    () => context.isSelected$(ref),
    [context, ref],
    context.isSelected(ref),
  )

  React.useEffect(() => {
    if (!ref) {
      return noop
    }

    const onActivate = () => {
      ref.click()
    }
    ref.addEventListener('activate', onActivate)

    const unregister =  context.registerNode(ref)

    return () => {
      ref.removeEventListener('activate', onActivate)

      unregister()
    }
  }, [ref])

  return [isFocused, setRef]
}
