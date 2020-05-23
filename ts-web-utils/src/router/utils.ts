import { isString } from '@v/utils'
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

export function updateParam(params: IQueryParam[], name: string, value: string | undefined) {
  return params.map(param => (
    param.name === name ? { name, value } : param
  ))
}

export function simpleLocation2Location(simpleLocation: SimpleLocation): ILocation {
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

export function stringifyQueryParams(params: IQueryParam[]): string {
  const queryParams = new URLSearchParams()
  for (const param of params) {
    if (param.value !== undefined) {
      queryParams.append(param.name, param.value)
    }
  }

  const result = queryParams.toString()

  if (result.length) {
    return '?' + result
  }

  return result
}
