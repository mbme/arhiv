import * as React from 'react'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'

export function useFocusable<T extends HTMLElement>(ref: React.RefObject<T>, disabled = false): boolean {
  const context = React.useContext(FocusManagerContext)
  if (!context) {
    throw new Error('FocusManager must be provided')
  }

  const [isSelected, setIsSelected] = React.useState(() => context.isNodeSelected(ref.current))

  React.useEffect(() => {
    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return context.isNodeSelected$(el).subscribe({
      next: setIsSelected,
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

    return context.registerNode(el)
  }, [ref, disabled])

  return isSelected
}
