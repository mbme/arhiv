import * as React from 'react'
import { noop, Procedure } from '@v/utils'
import { FocusManagerContext } from './context'

interface IOptions {
  disabled?: boolean
  onFocus?: Procedure
  autoFocus?: boolean
}

export function useFocusable<T extends HTMLElement>(ref: React.RefObject<T>, options: IOptions = {}): boolean {
  const focusManager = React.useContext(FocusManagerContext)

  const [isSelected, setIsSelected] = React.useState(false)

  React.useEffect(() => {
    if (!focusManager) {
      return noop
    }

    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return focusManager.isChildSelected$(el).subscribe({
      next: setIsSelected,
    })
  }, [focusManager])

  React.useEffect(() => {
    if (options.disabled || !focusManager) {
      return noop
    }

    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return focusManager.registerChild(el, options.autoFocus)
  }, [focusManager, options.disabled])

  React.useEffect(() => {
    if (options.onFocus && isSelected) {
      options.onFocus()
    }
  }, [options.onFocus, isSelected])

  return isSelected
}
