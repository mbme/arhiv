import * as React from 'react'

export function useTextareaController(ref: React.RefObject<HTMLTextAreaElement>) {
  const selectionRef = React.useRef([0, 0])

  React.useEffect(() => {
    const el = ref.current
    if (!el) {
      throw new Error('textarea dom element must be provided')
    }

    const onBlur = () => {
      selectionRef.current = [el.selectionStart, el.selectionEnd]
    }

    el.addEventListener('blur', onBlur)

    return () => {
      el.removeEventListener('blur', onBlur)
    }
  }, [])

  const insert = (str: string) => {
    const el = ref.current
    if (!el) {
      throw new Error('textarea dom element must be provided')
    }

    const value = el.value

    const [selectionStart, selectionEnd] = selectionRef.current

    el.value = `${value.substring(0, selectionStart)}${str}${value.substring(selectionEnd)}`

    const newSelectionStart = selectionStart + str.length
    const newSelectionEnd = selectionStart

    selectionRef.current = [newSelectionStart, newSelectionEnd]

    el.setSelectionRange(newSelectionStart, newSelectionEnd)
  }

  const focus = () => {
    const el = ref.current
    if (!el) {
      throw new Error('textarea dom element must be provided')
    }

    el.focus()
  }

  return {
    insert,
    focus,
  }
}
