import { isObject } from './type-asserts'
import { Obj } from './types'

export const isEmptyObject = (x: Obj) => Object.keys(x).length === 0

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

export function merge<T>(origVal: T, newVal: T): T {
  if (!isObject(origVal) || !isObject(newVal)) {
    return newVal
  }

  const result: Obj = { ...origVal }

  for (const [key, value] of Object.entries(newVal)) {
    result[key] = merge(result[key], value)
  }

  return result as T
}

export function createProxy<T extends Obj>(
  target: T,
  handler: (prop: string, target: T) => unknown,
) {
  return new Proxy(target, {
    get(_, prop: string) {
      return handler(prop, target)
    },
  })
}
