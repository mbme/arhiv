import { isString } from '~/utils'
import {
  QueryParamType,
  ILocation,
  SimpleLocation,
  IParams,
} from './types'

export function paramAsArray(param: QueryParamType): readonly string[] {
  if (!param) {
    return []
  }

  if (isString(param)) {
    return [param]
  }

  return param
}

export function paramAsString(param: QueryParamType): string {
  if (!param) {
    return ''
  }

  if (isString(param)) {
    return param
  }

  return param[0] || ''
}

function simpleLocation2Location(simpleLocation: SimpleLocation): ILocation {
  if (isString(simpleLocation)) {
    return {
      path: simpleLocation,
      params: {},
    }
  }

  return {
    path: simpleLocation.path,
    params: simpleLocation.params || {},
  }
}

export function getCurrentLocation(): ILocation {
  const location = new URL(document.location.toString())
  const params: IParams = {}

  location.searchParams.forEach((value, key) => {
    const previousValue = params[key]
    if (!previousValue) {
      params[key] = value
      return
    }

    if (isString(previousValue)) {
      params[key] = [previousValue, value]
      return
    }

    params[key] = [...previousValue, value]
  })

  return {
    path: location.pathname,
    params,
  }
}

export function getUrl(simpleLocation: SimpleLocation) {
  const location = simpleLocation2Location(simpleLocation)

  const queryParams = new URLSearchParams()
  for (const [key, value] of Object.entries(location.params)) {
    if (value === undefined) {
      continue
    }

    if (isString(value)) {
      queryParams.set(key, value)
      continue
    }

    for (const arrayItem of value) {
      queryParams.set(key, arrayItem)
      continue
    }
  }

  const paramsStr = queryParams.toString()

  const url = `${window.location.origin}${location.path}`

  if (!paramsStr) {
    return url
  }

  return `${url}?${paramsStr}`
}
