import * as React from 'react'
import { noop } from '@v/utils'

export function useClickOnActivate<T extends HTMLElement>(ref: React.RefObject<T>, disabled = false) {
  React.useEffect(() => {
    if (disabled) {
      return noop
    }

    const el = ref.current
    if (!el) {
      throw new Error('dom element must be available')
    }

    const onActivate = () => {
      el.click()
    }

    el.addEventListener('activate', onActivate)

    return () => {
      el.removeEventListener('activate', onActivate)
    }
  }, [disabled])
}
