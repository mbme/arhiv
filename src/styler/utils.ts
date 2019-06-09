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

export const hash2className = (hash: string) => `s-${hash}`
export const hash2class = (hash: string) => '.' + hash2className(hash)
