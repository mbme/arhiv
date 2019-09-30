import {
  createLogger,
  Callbacks,
} from '~/utils'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
} from '~/isodb/types'
import {
  LocalAttachments,
} from '~/isodb/replica'
import {
  Cell,
} from '~/utils/reactive'

const log = createLogger('arhiv:network-manager')

type NetworkState = 'online' | 'offline'
const readNetworkState = () => window.navigator.onLine ? 'online' : 'offline'

export class NetworkManager {
  networkState$: Cell<NetworkState>

  authorized$ = new Cell<boolean>(true)

  private _callbacks = new Callbacks()

  constructor() {
    this.networkState$ = new Cell(readNetworkState())

    const sendNetworkState = () => {
      this.networkState$.value = readNetworkState()
    }

    window.addEventListener('online', sendNetworkState)
    window.addEventListener('offline', sendNetworkState)

    this._callbacks.add(() => {
      window.removeEventListener('online', sendNetworkState)
      window.removeEventListener('offline', sendNetworkState)
    })

    this._callbacks.add(
      this.networkState$.value$.subscribe({
        next: value => log.info(`network gone ${value}`),
      }),
    )

    this._callbacks.add(
      this.authorized$.value$.subscribe({
        next: isAuthorized => log.info(`authorized: ${isAuthorized}`),
      }),
    )
  }

  authorize = async (password: string) => {
    this._assertIsOnline()

    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    if (response.ok) {
      this.authorized$.value = true
    } else {
      this._onNetworkError(response.status)
    }
  }

  deauthorize = () => {
    document.cookie = 'token=0; path=/'
    this.authorized$.value = false
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
    if (this.networkState$.value === 'offline') {
      throw new Error('Network is offline')
    }
  }

  private _assertAuthorized() {
    if (!this.authorized$.value) {
      throw new Error('Not authorized')
    }
  }

  private _onNetworkError(status: number) {
    log.warn(`network error, http status code ${status}`)

    if (status === 403) {
      this.authorized$.value = false
    }
  }

  isOnline() {
    return this.networkState$.value === 'online'
  }

  stop() {
    this._callbacks.runAll()
    log.debug('stopped')
  }
}
