import { createLogger } from '~/logger'
import { ReactiveValue } from '~/utils/reactive'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
} from '~/isodb/types'
import { LocalAttachments } from '~/isodb/replica'

const log = createLogger('arhiv:network-agent')

type NetworkState = 'online' | 'offline'

export class NetworkAgent {
  networkState: ReactiveValue<NetworkState>
  isAuthorized = new ReactiveValue(false)

  constructor() {
    this.networkState = new ReactiveValue<NetworkState>('online', (next) => {
      const onNetworkStateChange = () => {
        const newState = window.navigator.onLine ? 'online' : 'offline'
        next(newState)
        log.info(`network gone ${newState}`)
      }

      window.addEventListener('online', onNetworkStateChange)
      window.addEventListener('offline', onNetworkStateChange)

      return () => {
        window.removeEventListener('online', onNetworkStateChange)
        window.removeEventListener('offline', onNetworkStateChange)
      }
    })

    this.isAuthorized.subscribe((isAuthorized) => {
      log.info(`authrorized: ${isAuthorized}`)
    })
  }

  async authorize(password: string) {
    this._assertNetworkState()

    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    if (response.ok) {
      this.isAuthorized.next(true)
    } else {
      this._onNetworkError(response.status)
    }
  }

  deauthorize() {
    document.cookie = 'token=0; path=/'
    this.isAuthorized.next(false)
  }

  async syncChanges<T extends IDocument>(
    changeset: IChangeset,
    localAttachments: LocalAttachments,
  ): Promise<IChangesetResult<T>> {
    this._assertNetworkState()

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

  private _assertNetworkState() {
    if (this.networkState.currentValue === 'offline') {
      throw new Error('Network is offline')
    }
  }

  private _onNetworkError(status: number) {
    log.warn(`network error, http status code ${status}`)

    if (status === 403) {
      this.isAuthorized.next(false)
    }
  }

  isOnline() {
    return this.networkState.currentValue === 'online'
  }

  stop() {
    this.networkState.destroy()
    this.isAuthorized.destroy()
    log.debug('stopped')
  }
}
