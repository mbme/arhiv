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
