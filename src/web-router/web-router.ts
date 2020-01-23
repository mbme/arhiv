import { Cell } from '~/reactive'
import {
  ILocation,
  SimpleLocation,
  QueryParamType,
} from './types'
import {
  getCurrentLocation,
  getUrl,
} from './utils'

export class WebRouter {
  location$ = new Cell<ILocation>(getCurrentLocation())

  constructor() {
    window.addEventListener('popstate', this._propagateCurrentLocation)
  }

  private _propagateCurrentLocation = () => {
    this.location$.value = getCurrentLocation()
  }

  push(location: SimpleLocation) {
    window.history.pushState(undefined, '', getUrl(location))
    this._propagateCurrentLocation()
  }

  replace(location: SimpleLocation) {
    window.history.replaceState(undefined, '', getUrl(location))
    this._propagateCurrentLocation()
  }

  replaceParam(param: string, value: QueryParamType) {
    const {
      path,
      params,
    } = this.location$.value

    const newLocation: ILocation = {
      path,
      params: {
        ...params,
        [param]: value,
      },
    }

    window.history.replaceState(undefined, '', getUrl(newLocation))
    this._propagateCurrentLocation()
  }

  stop() {
    window.removeEventListener('popstate', this._propagateCurrentLocation)
  }
}
