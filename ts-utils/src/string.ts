/**
 * Check if needle fuzzy matches haystack.
 * @see https://github.com/bevacqua/fuzzysearch
 */
export function fuzzySearch(needle: string, haystack: string, ignoreCase = true): boolean {
  if (ignoreCase) {
    return fuzzySearch(needle.toLowerCase(), haystack.toLowerCase(), false)
  }

  const nlen = needle.length

  // if needle is empty then it matches everything
  if (!nlen) {
    return true
  }

  const hlen = haystack.length
  if (nlen > hlen) {
    return false
  }

  if (nlen === hlen) {
    return needle === haystack
  }

  // eslint-disable-next-line no-labels
  outer: for (let i = 0, j = 0; i < nlen; i += 1) {
    const nch = needle.charCodeAt(i)
    while (j < hlen) {
      const char = haystack.charCodeAt(j)

      j += 1

      if (char === nch) {
        continue outer // eslint-disable-line no-labels
      }
    }

    return false
  }

  return true
}

export function reverse(str: string) {
  return Array.from(str).reverse().join('')
}

export function trimLeft(str: string, chars = ' ') {
  if (!chars) {
    throw new Error('chars must not be empty ')
  }

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

export function trimPrefix(str: string, prefix: string) {
  if (!prefix) {
    throw new Error('prefix must not be empty ')
  }

  if (str.startsWith(prefix)) {
    return str.substring(prefix.length)
  }

  return str
}

export function trimSuffix(str: string, suffix: string) {
  if (!suffix) {
    throw new Error('suffix must not be empty ')
  }

  if (str.endsWith(suffix)) {
    return str.substring(0, str.length - suffix.length)
  }

  return str
}

// http://werxltd.com/wp/2010/05/13/javascript-implementation-of-javas-string-hashcode-method/
export function hashCode(str: string): number {
  let hash = 0

  for (let i = 0; i < str.length; i += 1) {
    // eslint-disable-next-line no-bitwise
    hash = ((hash << 5) - hash) + str.charCodeAt(i)

    // eslint-disable-next-line no-bitwise
    hash |= 0 // Convert to 32bit integer
  }

  return hash
}

const upperCasePattern = /[A-Z]/g
export const camelCase2kebabCase = (prop: string) => prop.replace(upperCasePattern, match => `-${match.toLowerCase()}`)

export const capitalize = (str: string) => str[0].toUpperCase() + str.substring(1)

export function isSubSequence(str: string, i: number, seq: string) {
  for (let pos = 0; pos < seq.length; pos += 1) {
    if (str[i + pos] !== seq[pos]) {
      return false
    }
  }

  return true
}

export const isSha256 = (str: string) => /^[a-f0-9]{64}$/i.test(str)

export function parseInt10(str: string) {
  return parseInt(str, 10)
}

function padString(input: string): string {
  const segmentLength = 4
  const stringLength = input.length
  const diff = stringLength % segmentLength

  if (!diff) {
    return input
  }

  let position = stringLength
  let padLength = segmentLength - diff
  const paddedStringLength = stringLength + padLength
  const buffer = Buffer.alloc(paddedStringLength)

  buffer.write(input)

  while (padLength--) {
    buffer.write('=', position++)
  }

  return buffer.toString()
}

// based on https://github.com/brianloveswords/base64url
export function base64url2base64(base64url: string): string {
  return padString(base64url)
    .replace(/-/g, '+')
    .replace(/_/g, '/')
}

export function countSubstring(str: string, substr: string): number {
  if (!substr.length) {
    throw new Error('substring must not be empty')
  }

  let count = 0

  let pos = 0
  while (pos < str.length) {
    if (isSubSequence(str, pos, substr)) {
      count += 1
    }

    pos += substr.length
  }

  return count
}
