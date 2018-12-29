// tslint:disable:max-line-length
import React, { PureComponent } from 'react'
import WebRouter, { IRoute } from '../web-router'
import { Omit, TypeOfProperty } from '../utils'
import IsodbClient from '../isodb/web-client'
import IsodbReplica from '../isodb/core/replica'
import * as CoreTypes from '../isodb/core/types'

export type Note = CoreTypes.INote

const StoreContext = React.createContext({} as any)

export interface IStoreState {
  route?: IRoute
  toast?: JSX.Element
  isLockerVisible: boolean
  isNavVisible: boolean
  isAuthorized: boolean
}

export class StoreProvider extends PureComponent<{}, IStoreState> {
  router = new WebRouter()
  client = new IsodbClient()

  state: IStoreState = {
    route: undefined,
    toast: undefined,
    isLockerVisible: false,
    isNavVisible: false,
    isAuthorized: true,
  }

  actions = {
    showToast: (toast: JSX.Element) => this.setState({ toast }),
    hideToast: () => this.setState({ toast: undefined }),
    showLocker: (show: boolean) => this.setState({ isLockerVisible: show }),
    showNav: (show: boolean) => this.setState({ isNavVisible: show }),
    push: (route: IRoute) => this.router.push(route),
    replace: (route: IRoute) => this.router.replace(route),
    replaceParam: (param: string, value: string) => this.router.replaceParam(param, value),
    authorize: (password: string) => this.client.authorize(password),
    deauthorize: () => this.client.deauthorize(),
  }

  _updateRoute = (route: IRoute) => this.setState({ route })
  _saveAuth = (isAuthorized: boolean) => this.setState({ isAuthorized })

  componentDidMount() {
    this.client.events.on('authorized', this._saveAuth)
    this.router.events.on('route-changed', this._updateRoute)
    this.router.start()
  }

  componentWillUnmount() {
    this.client.events.off('authorized', this._saveAuth)
    this.router.events.off('route-changed', this._updateRoute)
    this.router.stop()
  }

  render() {
    return (
      <StoreContext.Provider value={{ state: this.state, actions: this.actions, client: this.client }}>
        {this.props.children}
      </StoreContext.Provider>
    )
  }
}

export type ActionsType = TypeOfProperty<StoreProvider, 'actions'>
export { IsodbReplica }

// https://github.com/DefinitelyTyped/DefinitelyTyped/blob/93d063e00ef7eddb4d5ef5c910b5028d6fec6099/types/react-redux/index.d.ts#L75-L90
type Shared<
  InjectedProps,
  DecorationTargetProps extends Shared<InjectedProps, DecorationTargetProps>
  > = {
    [P in Extract<keyof InjectedProps, keyof DecorationTargetProps>]?: InjectedProps[P] extends DecorationTargetProps[P] ? DecorationTargetProps[P] : never;
  }

export function inject<PropsType, MappedPropsType>(
  mapStoreToProps: (state: IStoreState, actions: ActionsType, db: IsodbReplica) => MappedPropsType,
  Component: React.ComponentType<PropsType>
) {
  type InjectedPropsType = Shared<PropsType, MappedPropsType>
  type PropsWithoutInjectedPropsType = Omit<PropsType, keyof InjectedPropsType>

  interface IStoreContext {
    state: IStoreState,
    actions: ActionsType,
    client: IsodbClient,
  }

  // tslint:disable-next-line:max-classes-per-file
  return class StoreInjector extends PureComponent<PropsWithoutInjectedPropsType> {
    static contextType = StoreContext
    context!: React.ContextType<typeof StoreContext>

    _subscribed = false
    _onDBUpdate = () => {
      this.forceUpdate()
    }

    componentWillUnmount() {
      if (this._subscribed) {
        (this.context as IStoreContext).client.events.off('db-update', this._onDBUpdate)
      }
    }

    render() {
      const {
        state,
        actions,
        client,
      } = this.context as IStoreContext

      const mappedProps = mapStoreToProps(state, actions, client.db)

      if (!this._subscribed && mapStoreToProps.length === 3) {
        client.events.on('db-update', this._onDBUpdate)
        this._subscribed = true
      }

      return (
        <Component {...{ ...this.props, ...mappedProps } as any} />
      )
    }
  }
}
