import {
  createLogger,
  Callbacks,
} from '~/utils'
import {
  Cell,
} from '~/reactive'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
} from '../../types'
import {
  LocalAttachments,
} from '../types'

const log = createLogger('arhiv:network-manager')

const readNetworkState = () => window.navigator.onLine

export class NetworkManager {
  readonly isOnline$ = new Cell<boolean>(readNetworkState())
  readonly isAuthorized$ = new Cell<boolean>(true)

  private _callbacks = new Callbacks()

  constructor() {
    const sendNetworkState = () => {
      this.isOnline$.value = readNetworkState()
    }

    window.addEventListener('online', sendNetworkState)
    window.addEventListener('offline', sendNetworkState)

    this._callbacks.add(
      () => {
        window.removeEventListener('online', sendNetworkState)
        window.removeEventListener('offline', sendNetworkState)
      },
      this.isOnline$.value$.subscribe({
        next: value => log.info(`network is ${value ? 'online' : 'offline'}`),
      }),
      this.isAuthorized$.value$.subscribe({
        next: isAuthorized => log.info(`authorized: ${isAuthorized}`),
      }),
    )
  }

  async authorize(password: string) {
    this._assertIsOnline()

    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    if (response.ok) {
      this.isAuthorized$.value = true
    } else {
      this._onNetworkError(response.status)
    }
  }

  deauthorize() {
    document.cookie = 'token=0; path=/'
    this.isAuthorized$.value = false
  }

  readonly syncChanges = async <T extends IDocument>(
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
    if (!this.isOnline$.value) {
      throw new Error('Network is offline')
    }
  }

  private _assertAuthorized() {
    if (!this.isAuthorized$.value) {
      throw new Error('Not authorized')
    }
  }

  private _onNetworkError(status: number) {
    log.warn(`network error, http status code ${status}`)

    if (status === 403) {
      this.isAuthorized$.value = false
    }
  }

  isOnline() {
    return this.isOnline$.value
  }

  isAuthorized() {
    return this.isAuthorized$.value
  }

  stop() {
    this._callbacks.runAll(true)
    log.debug('stopped')
  }
}
