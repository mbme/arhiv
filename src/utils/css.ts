import {
  isString,
  isObject,
} from './type-asserts'

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
