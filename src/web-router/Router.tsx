import * as React from 'react'
import { OptionalProps } from '~/utils'
import {
  WebRouter,
  IRoute,
} from './router'

export type Route = OptionalProps<IRoute, 'params'>

interface IRouteActions {
  push(route: Route): void
  pushTo(path: string): void
  replace(route: Route): void
  replaceParam(param: string, value: string): void
  getUrl(route: Route): string
}

export const RouterContext = React.createContext<IRouteActions>({} as any)

interface IProps {
  renderView(route: IRoute): React.ReactNode
}

interface IState {
  view: React.ReactNode,
}

const normalizeRoute = (route: Route): IRoute => ({
  path: route.path,
  params: route.params || {},
})

export class Router extends React.PureComponent<IProps, IState> {
  state: IState = {
    view: null,
  }

  _router = new WebRouter()
  _updateRoute = (route: IRoute) => {
    this.setState({
      view: this.props.renderView(route),
    })
  }

  componentDidMount() {
    this._router.events.on('route-changed', this._updateRoute)
    this._router.start()
  }

  componentWillUnmount() {
    this._router.events.off('route-changed', this._updateRoute)
    this._router.stop()
  }

  actions: IRouteActions = {
    push: (route: Route) => {
      this._router.push(normalizeRoute(route))
    },

    pushTo: (path: string) => {
      this._router.push({ path, params: {} })
    },

    replace: (route: Route) => {
      this._router.replace(normalizeRoute(route))
    },

    replaceParam: (param: string, value: string) => {
      this._router.replaceParam(param, value)
    },

    getUrl: (route: Route) =>
      this._router.getUrl(normalizeRoute(route)),
  }

  render() {
    const {
      view,
    } = this.state

    return (
      <RouterContext.Provider value={this.actions}>
        {view}
      </RouterContext.Provider>
    )
  }
}
