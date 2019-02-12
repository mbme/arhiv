import React, { PureComponent } from 'react'
import {
  Omit,
  Shared,
} from '../../utils'
import IsodbReplica from '../../isodb/core/replica'
import AppStore from './app-store'

const StoreContext = React.createContext<AppStore>(null as any)

type PropsWithoutInjectedProps<Props, MappedProps> = Omit<Props, keyof Shared<Props, MappedProps>>

export function inject<Props, MappedProps>(
  // tslint:disable-next-line:max-line-length
  mapStoreToProps: (store: AppStore, props: PropsWithoutInjectedProps<Props, MappedProps>, db: IsodbReplica) => MappedProps,
  Component: React.ComponentType<Props>
) {
  const withDBSubscription = mapStoreToProps.length === 3

  return class extends PureComponent<PropsWithoutInjectedProps<Props, MappedProps>> {
    static displayName = `WithStore(${Component.displayName || Component.name || 'Component'})`

    static contextType = StoreContext
    context!: React.ContextType<typeof StoreContext>

    _onUpdate = () => {
      this.forceUpdate()
    }

    componentDidMount() {
      this.context.$state.on(this._onUpdate)

      if (withDBSubscription) {
        this.context._client.events.on('db-update', this._onUpdate)
      }
    }

    componentWillUnmount() {
      this.context.$state.off(this._onUpdate)

      if (withDBSubscription) {
        this.context._client.events.off('db-update', this._onUpdate)
      }
    }

    render() {
      const mappedProps = mapStoreToProps(this.context, this.props, this.context._client.db)

      return (
        <Component {...{ ...this.props, ...mappedProps } as any} />
      )
    }
  }
}

// tslint:disable-next-line:max-classes-per-file
export class StoreProvider extends PureComponent {
  store = new AppStore()

  componentDidMount() {
    this.store.start()
  }

  componentWillUnmount() {
    this.store.stop()
  }

  render() {
    return (
      <StoreContext.Provider value={this.store}>
        {this.props.children}
      </StoreContext.Provider>
    )
  }
}
