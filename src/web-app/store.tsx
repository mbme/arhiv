import React, { PureComponent } from 'react'
import WebRouter, { IRoute } from '../web-router'
import { Omit } from '../utils'

export interface IStore {
  route?: IRoute
  toast?: JSX.Element
  isLockerVisible: boolean
  isNavVisible: boolean
  isAuthorized?: boolean

  showToast: (toast: JSX.Element) => void
  showLocker: (show: boolean) => void
  showNav: (show: boolean) => void
  setAuthorized: (authorized: boolean) => void
  push: (route: IRoute) => void
  replace: (route: IRoute) => void
}

const StoreContext = React.createContext<IStore>({} as any)

export class StoreProvider extends PureComponent {
  state = {
    route: undefined,
    toast: undefined,
    isLockerVisible: false,
    isNavVisible: false,
    isAuthorized: undefined,
  }

  router = new WebRouter((route: IRoute) => this.setState({ route }))

  componentDidMount() {
    this.router.start()
  }
  componentWillUnmount() {
    this.router.stop()
  }

  showToast = (toast?: JSX.Element) => this.setState({ toast })
  showLocker = (show: boolean) => this.setState({ isLockerVisible: show })
  showNav = (show: boolean) => this.setState({ isNavVisible: show })
  setAuthorized = (authorized: boolean) => this.setState({ isAuthorized: authorized })

  push = (route: IRoute) => this.router.push(route)
  replace = (route: IRoute) => this.router.replace(route)

  render() {
    const store: IStore = {
      ...this.state,

      showToast: this.showToast,
      showLocker: this.showLocker,
      showNav: this.showNav,
      setAuthorized: this.setAuthorized,

      push: this.push,
      replace: this.replace,
    }

    return (
      <StoreContext.Provider value={store}>
        {this.props.children}
      </StoreContext.Provider>
    )
  }
}

export function inject<P extends PI, PI>(
  mapStoreToProps: (store: IStore) => PI,
  Component: React.ComponentType<P>
) {
  // tslint:disable-next-line:max-classes-per-file
  return class StoreInjector extends PureComponent<Omit<P, keyof PI>> {
    renderComponent = (store: IStore) => {
      const mappedProps = mapStoreToProps(store)

      return (
        <Component {...{ ...this.props, ...mappedProps }} />
      )
    }

    render() {
      return (
        <StoreContext.Consumer>
          {this.renderComponent}
        </StoreContext.Consumer>
      )
    }
  }
}
