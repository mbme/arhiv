import { isFunction } from './type-asserts'

export function createArray<T>(size: number, val: ((index: number) => T) | T): T[] {
  const arr = Array(size)

  return isFunction(val) ? arr.fill(0).map((_, i) => val(i)) : arr.fill(val)
}

export function uniq<T>(arr: T[], getKey: (item: T) => string) {
  const result: T[] = []
  const keys: string[] = []

  arr.forEach((item) => {
    const key = getKey(item)

    if (!keys.includes(key)) {
      result.push(item)
      keys.push(key)
    }
  })

  return result
}

export function removeMut<T>(arr: T[], value: T) {
  const pos = arr.indexOf(value)

  removeAtMut(arr, pos)

  return arr
}

export function removeAtMut<T>(arr: T[], pos: number) {
  if (pos > -1) {
    arr.splice(pos, 1)
  }

  return arr
}

export const findById = <T>(arr: Array<{ id: T }>, id: T) => arr.find((item) => item.id === id)

export function isEqualArray(a: any[], b: any[]) {
  if (a.length !== b.length) {
    return false
  }

  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) {
      return false
    }
  }

  return true
}

export function getLastEl<T>(arr: T[]): T {
  if (!arr.length) {
    throw new Error('array must have values')
  }

  return arr[arr.length - 1]
}
