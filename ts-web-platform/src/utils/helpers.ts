import * as React from 'react'
import { isFunction } from '@v/utils'

export function clickOnEnter(e: React.KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()

    // we use dispatchEvent here cause click() doesn't work on SVG elements
    // bubbles is required to make this work in React
    e.target.dispatchEvent(new Event('click', { bubbles: true }))
  }
}

// merge react refs
export function mergeRefs<T>(...refs: Array<React.Ref<T> | undefined>): React.Ref<T> | undefined {
  if (refs.filter(Boolean).length === 0) {
    return undefined
  }

  return (value) => {
    for (const ref of refs) {
      if (!ref) {
        continue
      }

      if (isFunction(ref)) {
        ref(value)
      } else {
        // eslint-disable-next-line
        (ref as any).current = value
      }
    }
  }
}

export function focusInput(input: HTMLInputElement) {
  input.focus()

  // put cursor at the end of the input
  const { length } = input.value
  input.setSelectionRange(length, length)
}
