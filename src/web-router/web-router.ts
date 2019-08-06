import { ReactiveValue } from '~/utils/reactive'

export interface IParams {
  [key: string]: string | undefined
}

export interface ILocation {
  path: string
  params: IParams
}

export class WebRouter {
  $location = new ReactiveValue<ILocation>(this._getCurrentLocation())

  constructor() {
    window.addEventListener('popstate', this._propagateCurrentLocation)
  }

  private _getCurrentLocation(): ILocation {
    const location = new URL(document.location.toString())
    const params: { [key: string]: string } = {}

    location.searchParams.forEach((value, key) => params[key] = value)

    return {
      path: location.pathname,
      params,
    }
  }

  private _propagateCurrentLocation = () => {
    this.$location.next(this._getCurrentLocation())
  }

  getUrl(location: ILocation) {
    const queryParams = new URLSearchParams()
    for (const [key, value] of Object.entries(location.params)) {
      if (value !== undefined) {
        queryParams.set(key, value)
      }
    }

    const paramsStr = queryParams.toString()

    const url = `${window.location.origin}${location.path}`

    if (!paramsStr) {
      return url
    }

    return `${url}?${paramsStr}`
  }

  push(location: ILocation) {
    window.history.pushState(undefined, '', this.getUrl(location))
    this._propagateCurrentLocation()
  }

  replace(location: ILocation) {
    window.history.replaceState(undefined, '', this.getUrl(location))
    this._propagateCurrentLocation()
  }

  replaceParam(param: string, value?: string) {
    const {
      path,
      params,
    } = this.$location.currentValue

    const newLocation: ILocation = {
      path,
      params: {
        ...params,
        [param]: value,
      },
    }

    window.history.replaceState(undefined, '', this.getUrl(newLocation))
    this._propagateCurrentLocation()
  }

  stop() {
    window.removeEventListener('popstate', this._propagateCurrentLocation)
    this.$location.destroy()
  }
}
