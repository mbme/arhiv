// tslint:disable:max-line-length
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
  hideToast: () => void
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

  showToast = (toast: JSX.Element) => this.setState({ toast })
  hideToast = () => this.setState({ toast: undefined })
  showLocker = (show: boolean) => this.setState({ isLockerVisible: show })
  showNav = (show: boolean) => this.setState({ isNavVisible: show })
  setAuthorized = (authorized: boolean) => this.setState({ isAuthorized: authorized })

  push = (route: IRoute) => this.router.push(route)
  replace = (route: IRoute) => this.router.replace(route)

  render() {
    const store: IStore = {
      ...this.state,

      showToast: this.showToast,
      hideToast: this.hideToast,
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

// https://github.com/DefinitelyTyped/DefinitelyTyped/blob/93d063e00ef7eddb4d5ef5c910b5028d6fec6099/types/react-redux/index.d.ts#L75-L90
type Shared<
  InjectedProps,
  DecorationTargetProps extends Shared<InjectedProps, DecorationTargetProps>
  > = {
    [P in Extract<keyof InjectedProps, keyof DecorationTargetProps>]?: InjectedProps[P] extends DecorationTargetProps[P] ? DecorationTargetProps[P] : never;
  }

export function inject<PropsType, MappedPropsType>(
  mapStoreToProps: (store: IStore) => MappedPropsType,
  Component: React.ComponentType<PropsType>
) {
  type InjectedPropsType = Shared<PropsType, MappedPropsType>
  type PropsWithoutInjectedPropsType = Omit<PropsType, keyof InjectedPropsType>

  // tslint:disable-next-line:max-classes-per-file
  return class StoreInjector extends PureComponent<PropsWithoutInjectedPropsType> {
    renderComponent = (store: IStore) => {
      const mappedProps = mapStoreToProps(store)

      return (
        <Component {...{ ...this.props, ...mappedProps } as any} />
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
