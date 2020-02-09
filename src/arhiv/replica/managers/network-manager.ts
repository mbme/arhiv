import { createLogger } from '~/logger'
import {
  Callbacks,
} from '~/utils'
import {
  Cell,
} from '~/reactive'
import {
  IChangeset,
  IChangesetResponse,
} from '~/arhiv/types'
import {
  LocalAttachments,
} from '../types'

const log = createLogger('arhiv:network-manager', 'greenBright')

const readNetworkState = () => window.navigator.onLine

export class NetworkManager {
  readonly isOnline$ = new Cell<boolean>(readNetworkState())

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
    )
  }

  readonly syncChanges = async (
    changeset: IChangeset,
    localAttachments: LocalAttachments,
  ): Promise<IChangesetResponse> => {
    this._assertIsOnline()

    const data = new FormData()
    data.append('changeset', JSON.stringify(changeset))
    for (const [id, blob] of Object.entries(localAttachments)) {
      data.append(id, blob)
    }

    const response = await fetch('/api/changeset', {
      method: 'post',
      credentials: 'include',
      body: data,
    }).catch((err) => {
      throw new Error(`Network error: ${err}`)
    })

    if (!response.ok) {
      throw new Error(`Server responded with code ${response.status}`)
    }

    return response.json()
  }

  private _assertIsOnline() {
    if (!this.isOnline$.value) {
      throw new Error('Network is offline')
    }
  }

  isOnline() {
    return this.isOnline$.value
  }

  stop() {
    this._callbacks.runAll(true)
    log.debug('stopped')
  }
}
