export const getType = (x: any) => Object.prototype.toString.call(x).slice(8, -1)

export const isObject = (x: any) => getType(x) === 'Object'
export const isArray = (x: any) => getType(x) === 'Array'
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

/**
 * Create new object with specified prototype `proto` and custom `props`
 */
export function extend(proto: object, props: { [key: string]: any }): object {
  const propertiesObject: { [key: string]: any } = {}

  Object.keys(props).forEach((prop) => {
    propertiesObject[prop] = { value: props[prop] }
  })

  return Object.create(proto, propertiesObject)
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
export function flatten(arr: any[]) {
  return arr.reduce((acc, item) => {
    if (isArray(item)) {
      acc.push(...item)
    } else {
      acc.push(item)
    }
    return acc
  }, [])
}

export const isSha256 = (str: string) => /^[a-f0-9]{64}$/i.test(str)

export function isSubSequence(str: string, i: number, seq: string) {
  for (let pos = 0; pos < seq.length; pos += 1) {
    if (str[i + pos] !== seq[pos]) return false
  }

  return true
}

export function formatTs(ts: number) {
  const date = new Date(ts)

  return [
    date.getFullYear(),
    date.getMonth() + 1,
    date.getDate(),
  ].join('/')
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

export function promiseTimeout(timeout: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, timeout))
}

export function createProxy<T extends object>(target: T, handler: (prop: string, target: T) => any) {
  return new Proxy(target, {
    get(_, prop: string) {
      return handler(prop, target)
    },
  })
}

export interface ILazy<T> {
  readonly initialized: boolean
  readonly value: T
}
export function lazy<T>(createVal: () => T): ILazy<T> {
  let val: T
  let initialized = false

  return {
    get initialized() {
      return initialized
    },
    get value(): T {
      if (!val) {
        initialized = true
        val = createVal()
      }

      return val
    },
  }
}

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

export type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>

export type Merge<L, R> =
  // Mandatory properties
  Pick<L, Exclude<keyof L, keyof R>>
  // Optional properties
  & Partial<R>
