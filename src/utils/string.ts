/**
 * Check if needle fuzzy matches haystack.
 * @see https://github.com/bevacqua/fuzzysearch
 */
export function fuzzySearch(needle: string, haystack: string, ignoreCase = true): boolean {
  if (ignoreCase) return fuzzySearch(needle.toLowerCase(), haystack.toLowerCase(), false)

  const nlen = needle.length

  // if needle is empty then it matches everything
  if (!nlen) return true

  const hlen = haystack.length
  if (nlen > hlen) return false

  if (nlen === hlen) return needle === haystack

  outer: for (let i = 0, j = 0; i < nlen; i += 1) {
    const nch = needle.charCodeAt(i)
    while (j < hlen) {
      // tslint:disable-next-line:increment-decrement
      if (haystack.charCodeAt(j++) === nch) continue outer
    }

    return false
  }

  return true
}

export function reverse(str: string) {
  return Array.from(str).reverse().join('')
}

export function trimLeft(str: string, chars = ' ') {
  if (!chars) throw new Error('chars must not be empty ')

  for (let i = 0; i < str.length; i += 1) {
    if (!chars.includes(str[i])) {
      return str.substring(i)
    }
  }

  return ''
}

export function trimRight(str: string, chars = ' ') {
  return reverse(trimLeft(reverse(str), chars))
}

export function trim(str: string, chars = ' ') {
  return trimRight(trimLeft(str, chars), chars)
}
