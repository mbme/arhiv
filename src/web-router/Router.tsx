import * as React from 'react'
import {
  WebRouter,
  IRoute,
} from './router'

interface IRouteActions {
  push(route: IRoute): void
  pushTo(path: string): void
  replace(route: IRoute): void
  replaceParam(param: string, value: string): void
}

export const RouterContext = React.createContext<IRouteActions>({} as any)

interface IProps {
  renderView(route: IRoute): React.ReactNode
}

interface IState {
  route?: IRoute,
}

export class Router extends React.PureComponent<IProps, IState> {
  state: IState = {
    route: undefined,
  }

  _router = new WebRouter()
  _updateRoute = (route: IRoute) => this.setState({ route })

  componentDidMount() {
    this._router.events.on('route-changed', this._updateRoute)
    this._router.start()
  }

  componentWillUnmount() {
    this._router.events.off('route-changed', this._updateRoute)
    this._router.stop()
  }

  actions: IRouteActions = {
    push: (route: IRoute) => this._router.push(route),
    pushTo: (path: string) => this._router.push({ path, params: {} }),
    replace: (route: IRoute) => this._router.replace(route),
    replaceParam: (param: string, value: string) => this._router.replaceParam(param, value),
  }

  render() {
    const {
      renderView,
    } = this.props

    const {
      route,
    } = this.state

    if (!route) {
      return null
    }

    return (
      <RouterContext.Provider value={this.actions}>
        {renderView(route)}
      </RouterContext.Provider>
    )
  }
}
