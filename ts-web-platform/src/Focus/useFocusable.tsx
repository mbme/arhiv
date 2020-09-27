import * as React from 'react'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'

export function useFocusable<T extends HTMLElement>(ref: React.RefObject<T>, disabled = false): boolean {
  const context = React.useContext(FocusManagerContext)
  if (!context) {
    throw new Error('FocusManager must be provided')
  }

  const [isSelected, setIsFocused] = React.useState(() => context.isSelected(ref.current))

  React.useEffect(() => {
    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return context.isSelected$(el).subscribe({
      next: setIsFocused,
    })
  }, [])

  React.useEffect(() => {
    if (disabled) {
      return noop
    }
    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    const onActivate = () => {
      el.click()
    }
    el.addEventListener('activate', onActivate)

    const unregister =  context.registerNode(el)

    return () => {
      el.removeEventListener('activate', onActivate)

      unregister()
    }
  }, [ref, disabled])

  return isSelected
}
