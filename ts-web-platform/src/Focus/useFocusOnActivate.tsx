import * as React from 'react'

export function useFocusOnActivate<T extends HTMLElement>(ref: React.RefObject<T>) {
  React.useEffect(() => {
    const el = ref.current
    if (!el) {
      throw new Error('dom element must be available')
    }

    const onActivate = () => {
      el.focus()
    }

    el.addEventListener('activate', onActivate)

    return () => {
      el.removeEventListener('activate', onActivate)
    }
  }, [])
}
