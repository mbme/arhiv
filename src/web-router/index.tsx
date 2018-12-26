import PubSub from '../utils/pubsub'

export interface IRoute {
  path: string
  params: { [key: string]: string }
}

export interface IEvents {
  'route-changed': IRoute
}

export default class WebRouter {
  route?: IRoute

  constructor(public events = new PubSub<IEvents>()) { }

  _createUrl(route: IRoute) {
    const queryParams = new URLSearchParams()
    Object.entries(route.params).forEach(([key, value]) => {
      queryParams.set(key, value)
    })
    const paramsStr = queryParams.toString()

    const url = `${window.location.origin}/${name}`

    if (!paramsStr) {
      return url
    }

    return `${url}?${paramsStr}`
  }

  _propagateCurrentLocation = () => {
    const location = new URL(document.location.toString())
    const params: { [key: string]: string } = {}
    for (const [key, value] of (location.searchParams as any)) {
      params[key] = value
    }

    this.route = {
      path: location.pathname,
      params,
    }
    this.events.emit('route-changed', this.route)
  }

  push(route: IRoute) {
    window.history.pushState(undefined, '', this._createUrl(route))
    this._propagateCurrentLocation()
  }

  replace(route: IRoute) {
    window.history.replaceState(undefined, '', this._createUrl(route))
    this._propagateCurrentLocation()
  }

  replaceParam(param: string, value: string) {
    if (!this.route) throw new Error('not started yet')

    const newRoute: IRoute = {
      path: this.route.path,
      params: {
        ...this.route.params,
        [param]: value,
      },
    }

    window.history.replaceState(undefined, '', this._createUrl(newRoute))
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
