import { PubSub } from '~/utils'

export interface IParams {
  [key: string]: string | undefined
}

export interface ILocation {
  path: string
  params: IParams
}

export interface IEvents {
  'location-changed': ILocation
}

export class WebRouter {
  location?: ILocation

  constructor(public events = new PubSub<IEvents>()) { }

  _propagateCurrentLocation = () => {
    const location = new URL(document.location.toString())
    const params: { [key: string]: string } = {}

    location.searchParams.forEach((value, key) => params[key] = value)

    this.location = {
      path: location.pathname,
      params,
    }
    this.events.emit('location-changed', this.location)
  }

  getUrl(location: ILocation) {
    const queryParams = new URLSearchParams()
    Object.entries(location.params).forEach(([key, value]) => {
      if (value !== undefined) {
        queryParams.set(key, value)
      }
    })
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
    if (!this.location) throw new Error('not started yet')

    const newLocation: ILocation = {
      path: this.location.path,
      params: {
        ...this.location.params,
        [param]: value,
      },
    }

    window.history.replaceState(undefined, '', this.getUrl(newLocation))
    this._propagateCurrentLocation()
  }

  start() {
    window.addEventListener('popstate', this._propagateCurrentLocation)
    this._propagateCurrentLocation()
  }

  stop() {
    window.removeEventListener('popstate', this._propagateCurrentLocation)
  }
}
