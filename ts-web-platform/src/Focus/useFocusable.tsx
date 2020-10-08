import * as React from 'react'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'

// TODO select child in Focus Manager if it was focused
export function useFocusable<T extends HTMLElement>(ref: React.RefObject<T>, disabled = false): boolean {
  const context = React.useContext(FocusManagerContext)

  const [isSelected, setIsSelected] = React.useState(false)

  React.useEffect(() => {
    if (!context) {
      return noop
    }

    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return context.isChildSelected$(el).subscribe({
      next: setIsSelected,
    })
  }, [context])

  React.useEffect(() => {
    if (disabled || !context) {
      return noop
    }

    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return context.registerChild(el)
  }, [context, disabled])

  return isSelected
}
