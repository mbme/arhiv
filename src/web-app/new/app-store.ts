import Store from './store'
import WebRouter, { IRoute } from '../../web-router'
import IsodbClient from '../../isodb/web-client'

interface IState {
  route?: IRoute
  toast?: JSX.Element
  isLockerVisible: boolean
  isNavVisible: boolean
  isAuthorized: boolean
}

export default class AppStore extends Store<IState> {
  router = new WebRouter()
  client = new IsodbClient()

  constructor() {
    super({
      route: undefined,
      toast: undefined,
      isLockerVisible: false,
      isNavVisible: false,
      isAuthorized: true,
    })
  }

  // router actions
  push = (route: IRoute) => this.router.push(route)
  replace = (route: IRoute) => this.router.replace(route)
  replaceParam = (param: string, value: string) => this.router.replaceParam(param, value)

  // auth actions
  authorize = (password: string) => this.client.authorize(password)
  deauthorize = () => this.client.deauthorize()

  // other
  showToast = (toast: JSX.Element) => this.setState({ toast })
  hideToast = () => this.setState({ toast: undefined })
  showLocker = (show: boolean) => this.setState({ isLockerVisible: show })
  showNav = (show: boolean) => this.setState({ isNavVisible: show })

  // helpers
  _updateRoute = (route: IRoute) => this.setState({ route })
  _saveAuth = (isAuthorized: boolean) => this.setState({ isAuthorized })

  start() {
    this.client.events.on('authorized', this._saveAuth)
    this.router.events.on('route-changed', this._updateRoute)

    this.client.start()
    this.router.start()
  }

  stop() {
    this.client.events.off('authorized', this._saveAuth)
    this.router.events.off('route-changed', this._updateRoute)

    this.router.stop()
    this.client.stop()
  }
}
