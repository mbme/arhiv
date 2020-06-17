/* eslint-disable @typescript-eslint/ban-types */

export const getType = (x: any) => Object.prototype.toString.call(x).slice(8, -1)

export const isObject = (x: any): x is object => getType(x) === 'Object'

export const isArray = <T>(x: any): x is T[] => getType(x) === 'Array'

export const isString = (x: any): x is string => getType(x) === 'String'

export const isFunction = (x: any): x is Function => ['Function', 'AsyncFunction'].includes(getType(x))

export const isAsyncFunction = (x: any): x is Function => getType(x) === 'AsyncFunction'
