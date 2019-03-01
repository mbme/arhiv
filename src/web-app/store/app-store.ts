import Store from '~/utils/store'
import WebRouter, { IRoute } from '~/web-router'
import IsodbClient from '~/isodb-web-client'

interface IState {
  route?: IRoute
  toast?: JSX.Element
  isLockerVisible: boolean
  isNavVisible: boolean
  isAuthorized: boolean
}

export default class AppStore extends Store<IState> {
  _router = new WebRouter()
  _client = new IsodbClient()

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
  push = (route: IRoute) => this._router.push(route)
  pushTo = (path: string) => this._router.push({ path, params: {} })
  replace = (route: IRoute) => this._router.replace(route)
  replaceParam = (param: string, value: string) => this._router.replaceParam(param, value)

  // auth actions
  authorize = (password: string) => this._client.authorize(password)
  deauthorize = () => this._client.deauthorize()

  // other
  showToast = (toast: JSX.Element) => this.setState({ toast })
  hideToast = () => this.setState({ toast: undefined })
  showLocker = (show: boolean) => this.setState({ isLockerVisible: show })
  showNav = (show: boolean) => this.setState({ isNavVisible: show })

  // manipulate records
  deleteRecord = (id: string) => {
    this._client.lockRecord(id)
    this._client.db.updateRecord(id, { _deleted: true })
    // FIXME locks
  }

  // helpers
  _updateRoute = (route: IRoute) => this.setState({ route })
  _saveAuth = (isAuthorized: boolean) => this.setState({ isAuthorized })

  start() {
    this._client.events.on('authorized', this._saveAuth)
    this._router.events.on('route-changed', this._updateRoute)

    this._client.start()
    this._router.start()
  }

  stop() {
    this._client.events.off('authorized', this._saveAuth)
    this._router.events.off('route-changed', this._updateRoute)

    this._router.stop()
    this._client.stop()
  }
}
