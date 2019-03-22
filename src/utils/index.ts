export { Counter } from './counter'

export const getType = (x: any) => Object.prototype.toString.call(x).slice(8, -1)

export const isObject = (x: any): x is object => getType(x) === 'Object'
export const isArray = <T>(x: any): x is T[] => getType(x) === 'Array'
export const isString = (x: any): x is string => getType(x) === 'String'
// tslint:disable-next-line:ban-types
export const isFunction = (x: any): x is Function => ['Function', 'AsyncFunction'].includes(getType(x))
// tslint:disable-next-line:ban-types
export const isAsyncFunction = (x: any): x is Function => getType(x) === 'AsyncFunction'

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

export const capitalize = (str: string) => str[0].toUpperCase() + str.substring(1)

export function createArray<T>(size: number, val: ((index: number) => T) | T): T[] {
  const arr = Array(size)

  return isFunction(val) ? arr.fill(0).map((_, i) => val(i)) : arr.fill(val)
}

export function uniq<T>(arr: T[], getKey: (item: T) => string = (val) => val.toString()) {
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
  if (pos > -1) {
    arr.splice(pos, 1)
  }

  return arr
}

export const findById = <T>(arr: Array<{ id: T }>, id: T) => arr.find((item) => item.id === id)

// [ [ 1, 2 ], 3 ] => [ 1, 2, 3 ]
export function flatten<T>(arr: Array<T | T[]>): T[] {
  const result = []
  for (const item of arr) {
    if (isArray<T>(item)) {
      result.push(...item)
    } else {
      result.push(item)
    }
  }

  return result
}

export const isSha256 = (str: string) => /^[a-f0-9]{64}$/i.test(str)

export function isSubSequence(str: string, i: number, seq: string) {
  for (let pos = 0; pos < seq.length; pos += 1) {
    if (str[i + pos] !== seq[pos]) return false
  }

  return true
}

export function mapObject<T, V>(obj: { [key: string]: T }, fn: (value: T, key: string) => V) {
  const result: { [key: string]: V } = {}

  for (const [key, value] of Object.entries(obj)) {
    result[key] = fn(value, key)
  }

  return result
}

export function array2object<T>(array: T[], getKey: (value: T) => string) {
  const result: { [key: string]: T } = {}

  for (const item of array) {
    result[getKey(item)] = item
  }

  return result
}

export function entries2object<V>(entries: Array<[string, V]>): { [key: string]: V } {
  const result: { [key: string]: V } = {}
  for (const [key, value] of entries) {
    result[key] = value
  }

  return result
}

export function map2object<V>(map: Map<string, V>) {
  const result: { [key: string]: V } = {}
  for (const [key, value] of map.entries()) {
    result[key] = value
  }

  return result
}

export async function promiseTimeout(timeout: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, timeout))
}

export function createProxy<T extends object>(target: T, handler: (prop: string, target: T) => any) {
  return new Proxy(target, {
    get(_, prop: string) {
      return handler(prop, target)
    },
  })
}

// tslint:disable-next-line:no-empty
export const noop = () => { }

export const nowS = () => Date.now() / 1000

export function classNames(...args: any[]) {
  return args.reduce<string[]>((acc, val) => {
    if (isString(val)) {
      acc.push(val)
    } else if (isObject(val)) {
      Object.entries(val).forEach(([key, assertion]) => {
        if (assertion) acc.push(key)
      })
    }

    return acc
  }, []).join(' ')
}

// Pick all props from type T except enumerated in K
export type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>

// Make props enumerated in TOptional optional
export type OptionalProps<T, TOptional extends keyof T> = Omit<T, TOptional> & Partial<Pick<T, TOptional>>

// Get type of object/class property
export type TypeOfProperty<T, P extends keyof T> = T[P]

// tslint:disable-next-line:max-line-length
// https://github.com/DefinitelyTyped/DefinitelyTyped/blob/93d063e00ef7eddb4d5ef5c910b5028d6fec6099/types/react-redux/index.d.ts#L75-L90
// Get shared props
export type Shared<A, B extends Shared<A, B>> = {
  [P in Extract<keyof A, keyof B>]?: A[P] extends B[P] ? B[P] : never;
}
