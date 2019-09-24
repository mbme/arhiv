import {
  createLogger,
} from '~/utils'
import {
  ReactiveValue,
} from '~/utils/reactive'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
} from '~/isodb/types'
import {
  LocalAttachments,
} from '~/isodb/replica'

const log = createLogger('arhiv:network-manager')

function monitorNetworkState$() {
  type NetworkState = 'online' | 'offline'

  const readNetworkState = () => window.navigator.onLine ? 'online' : 'offline'

  return new ReactiveValue<NetworkState>(readNetworkState(), (observer) => {
    const sendNetworkState = () => observer.next(readNetworkState())

    window.addEventListener('online', sendNetworkState)
    window.addEventListener('offline', sendNetworkState)

    return () => {
      window.removeEventListener('online', sendNetworkState)
      window.removeEventListener('offline', sendNetworkState)
    }
  })
}

export class NetworkManager {
  networkState$ = monitorNetworkState$().tap(
    value => log.info(`network gone ${value}`),
  )
  authorized$ = new ReactiveValue(true).tap(
    isAuthorized => log.info(`authorized: ${isAuthorized}`),
  )

  authorize = async (password: string) => {
    this._assertIsOnline()

    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    if (response.ok) {
      this.authorized$.next(true)
    } else {
      this._onNetworkError(response.status)
    }
  }

  deauthorize = () => {
    document.cookie = 'token=0; path=/'
    this.authorized$.next(false)
  }

  syncChanges = async <T extends IDocument>(
    changeset: IChangeset<T>,
    localAttachments: LocalAttachments,
  ): Promise<IChangesetResult<T>> => {
    this._assertIsOnline()
    this._assertAuthorized()

    const data = new FormData()
    data.append('changeset', JSON.stringify(changeset))
    for (const [id, blob] of Object.entries(localAttachments)) {
      data.append(id, blob)
    }

    const response = await fetch('/api/changeset', {
      method: 'post',
      credentials: 'include',
      body: data,
    })

    if (!response.ok) {
      this._onNetworkError(response.status)
      throw new Error(`Server responded with code ${response.status}`)
    }

    return response.json()
  }

  private _assertIsOnline() {
    if (this.networkState$.currentValue === 'offline') {
      throw new Error('Network is offline')
    }
  }

  private _assertAuthorized() {
    if (!this.authorized$.currentValue) {
      throw new Error('Not authorized')
    }
  }

  private _onNetworkError(status: number) {
    log.warn(`network error, http status code ${status}`)

    if (status === 403) {
      this.authorized$.next(false)
    }
  }

  isOnline() {
    return this.networkState$.currentValue === 'online'
  }

  stop() {
    this.networkState$.complete()
    this.authorized$.complete()
    log.debug('stopped')
  }
}
