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
        if (assertion) {
          acc.push(key)
        }
      })
    }

    return acc
  }, []).join(' ')
}

// helps to debug layout https://dev.to/gajus/my-favorite-css-hack-32g3
export const debugLayoutSnippet = `
  html * {
    background: rgba(255, 0, 0, .1);
    box-shadow: 0 0 0 1px red;
  }
`
