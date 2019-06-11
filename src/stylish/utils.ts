export function createStyleElement(prepend = false) {
  const el = document.createElement('style')
  el.setAttribute('type', 'text/css')

  if (prepend) {
    document.head.prepend(el)
  } else {
    document.head.appendChild(el)
  }

  return el
}

export function injectGlobalStyles(styles: string) {
  const el = createStyleElement(true)
  el.textContent = styles
}
