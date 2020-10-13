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
  if (!focusManager) {
    throw new Error('Focus Manager must be available')
  }

  const [isSelected, setIsSelected] = React.useState(false)

  React.useEffect(() => {
    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return focusManager.isChildSelected$(el).subscribe({
      next: setIsSelected,
    })
  }, [])

  React.useEffect(() => {
    if (options.disabled) {
      return noop
    }

    const el = ref.current
    if (!el) {
      throw new Error('dom element must be provided')
    }

    return focusManager.registerChild(el, options.autoFocus)
  }, [options.disabled])

  React.useEffect(() => {
    if (options.onFocus && isSelected) {
      options.onFocus()
    }
  }, [options.onFocus, isSelected])

  return isSelected
}
