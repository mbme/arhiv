import * as React from 'react'
import { OptionalProps } from '~/utils'
import {
  WebRouter,
  ILocation,
} from './router'

export type Location = OptionalProps<ILocation, 'params'>

interface IRouteActions {
  push(location: Location): void
  pushTo(path: string): void
  replace(location: Location): void
  replaceParam(param: string, value?: string): void
  getUrl(location: Location): string
}

export const RouterContext = React.createContext<IRouteActions>({} as any)

interface IProps {
  renderView(location: ILocation): React.ReactNode
}

interface IState {
  view: React.ReactNode,
}

const normalizeLocation = (location: Location): ILocation => ({
  path: location.path,
  params: location.params || {},
})

export class Router extends React.PureComponent<IProps, IState> {
  state: IState = {
    view: null,
  }

  _router = new WebRouter()
  _updateLocation = (location: ILocation) => {
    this.setState({
      view: this.props.renderView(location),
    })
  }

  componentDidMount() {
    this._router.events.on('location-changed', this._updateLocation)
    this._router.start()
  }

  componentWillUnmount() {
    this._router.events.off('location-changed', this._updateLocation)
    this._router.stop()
  }

  actions: IRouteActions = {
    push: (location: Location) => {
      this._router.push(normalizeLocation(location))
    },

    pushTo: (path: string) => {
      this._router.push({ path, params: {} })
    },

    replace: (location: Location) => {
      this._router.replace(normalizeLocation(location))
    },

    replaceParam: (param: string, value?: string) => {
      this._router.replaceParam(param, value)
    },

    getUrl: (location: Location) =>
      this._router.getUrl(normalizeLocation(location)),
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
