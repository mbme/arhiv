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
}

export const RouterContext = React.createContext<IRouteActions>({} as any)

interface IProps {
  renderView(route: IRoute): React.ReactNode
}

interface IState {
  view: React.ReactNode,
}

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
      this._router.push({
        path: route.path,
        params: route.params || {},
      })
    },

    pushTo: (path: string) => this._router.push({ path, params: {} }),

    replace: (route: Route) => {
      this._router.replace({
        path: route.path,
        params: route.params || {},
      })
    },

    replaceParam: (param: string, value: string) => this._router.replaceParam(param, value),
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
