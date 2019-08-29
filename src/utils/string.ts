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

// http://werxltd.com/wp/2010/05/13/javascript-implementation-of-javas-string-hashcode-method/
export function hashCode(str: string): number {
  let hash = 0

  for (let i = 0; i < str.length; i += 1) {
    // tslint:disable-next-line:no-bitwise
    hash = ((hash << 5) - hash) + str.charCodeAt(i)

    // tslint:disable-next-line:no-bitwise
    hash |= 0 // Convert to 32bit integer
  }

  return hash
}

const upperCasePattern = /[A-Z]/g
export const camelCase2kebabCase = (prop: string) => prop.replace(upperCasePattern, match => `-${match.toLowerCase()}`)

export const capitalize = (str: string) => str[0].toUpperCase() + str.substring(1)

export function isSubSequence(str: string, i: number, seq: string) {
  for (let pos = 0; pos < seq.length; pos += 1) {
    if (str[i + pos] !== seq[pos]) return false
  }

  return true
}

export const isSha256 = (str: string) => /^[a-f0-9]{64}$/i.test(str)
