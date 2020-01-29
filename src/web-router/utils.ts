import { isString } from '~/utils'
import {
  IQueryParam,
  ILocation,
  SimpleLocation,
} from './types'

export function paramAsArray(params: IQueryParam[], name: string): readonly string[] {
  return params.filter(param => param.name === name).map(param => param.value || '')
}

export function paramAsString(params: IQueryParam[], name: string): string {
  return params.find(param => param.name === name)?.value || ''
}

function simpleLocation2Location(simpleLocation: SimpleLocation): ILocation {
  if (isString(simpleLocation)) {
    return {
      path: simpleLocation,
      params: [],
    }
  }

  return {
    path: simpleLocation.path,
    params: simpleLocation.params || [],
  }
}

export function getCurrentLocation(): ILocation {
  const location = new URL(document.location.toString())
  const params: IQueryParam[] = []

  location.searchParams.forEach((value, key) => {
    params.push({
      name: key,
      value,
    })
  })

  return {
    path: location.pathname,
    params,
  }
}

export function getUrl(simpleLocation: SimpleLocation) {
  const location = simpleLocation2Location(simpleLocation)

  const queryParams = new URLSearchParams()
  for (const param of location.params) {
    if (param.value !== undefined) {
      queryParams.set(param.name, param.value)
    }
  }

  const paramsStr = queryParams.toString()

  const url = `${window.location.origin}${location.path}`

  if (!paramsStr) {
    return url
  }

  return `${url}?${paramsStr}`
}
