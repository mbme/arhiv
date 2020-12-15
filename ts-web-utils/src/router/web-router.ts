import { Cell } from '@v/reactive'
import {
  ILocation,
  SimpleLocation,
  IQueryParam,
} from './types'
import {
  simpleLocation2Location,
  stringifyQueryParams,
} from './utils'

function getHashLocation() {
  const queryPos = window.location.hash.indexOf('?')

  const endPos = queryPos === -1 ? window.location.hash.length : queryPos

  const pathname = window.location.hash.substring(1, endPos) || '/'
  const query = queryPos === -1 ? '' : window.location.hash.substring(queryPos, window.location.hash.length)

  return {
    pathname,
    query,
  }
}

export class WebRouter {
  locationRaw$ = new Cell<string>(this._getCurrentRawLocation())
  location$ = new Cell<ILocation>(this._getCurrentLocation())

  constructor(private _hashBased = false) {
    window.addEventListener('popstate', this._propagateCurrentLocation)
  }

  private _getCurrentRawLocation(): string {
    return document.location.toString()
  }

  private _getCurrentLocation(): ILocation {
    let path = ''
    const params: IQueryParam[] = []

    if (this._hashBased) {
      const {
        pathname,
        query,
      } = getHashLocation()

      path = pathname

      const searchParams = new URLSearchParams(query)
      searchParams.forEach((value, key) => {
        params.push({
          name: key,
          value,
        })
      })
    } else {
      const location = new URL(document.location.toString())

      path = location.pathname
      location.searchParams.forEach((value, key) => {
        params.push({
          name: key,
          value,
        })
      })
    }

    return {
      path,
      params,
    }
  }

  private _propagateCurrentLocation = () => {
    this.locationRaw$.value = this._getCurrentRawLocation()
    this.location$.value = this._getCurrentLocation()
  }

  getUrl(simpleLocation: SimpleLocation) {
    const location = simpleLocation2Location(simpleLocation)

    const paramsStr = stringifyQueryParams(location.params)

    if (this._hashBased) {
      const base = window.location.href.replace(/#(.*)$/, '')

      return `${base}#${location.path}${paramsStr}`
    }

    return `${window.location.origin}${location.path}${paramsStr}`
  }

  push(location: SimpleLocation) {
    const url = this.getUrl(location)

    window.history.pushState(undefined, '', url)
    this._propagateCurrentLocation()
  }

  replace(location: SimpleLocation) {
    const url = this.getUrl(location)

    window.history.replaceState(undefined, '', url)
    this._propagateCurrentLocation()
  }

  replaceParams(params: IQueryParam[]) {
    const {
      path,
    } = this.location$.value

    const newLocation: ILocation = {
      path,
      params,
    }

    const url = this.getUrl(newLocation)

    window.history.replaceState(undefined, '', url)
    this._propagateCurrentLocation()
  }

  goBack(fallback: SimpleLocation = { path: '/' }) {
    if (window.history.length > 1) {
      window.history.back()
    } else {
      this.push(fallback)
    }
  }

  stop() {
    window.removeEventListener('popstate', this._propagateCurrentLocation)
  }
}
