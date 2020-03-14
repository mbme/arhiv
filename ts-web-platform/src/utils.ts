import * as React from 'react'

export function clickOnEnter(e: React.KeyboardEvent) {
  if (e.key === 'Enter') {
    // we use dispatchEvent here cause click() doesn't work on SVG elements
    // bubbles is required to make this work in React
    e.target.dispatchEvent(new Event('click', { bubbles: true }))
  }
}
