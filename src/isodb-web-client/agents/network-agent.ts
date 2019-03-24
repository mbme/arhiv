import { createLogger } from '~/logger'
import {
  IRecord,
  IAttachment,
  IChangesetResult,
  IChangeset,
} from '~/isodb-core/types'
import { WebClientEvents } from '../events'

const log = createLogger('isodb-web-client:network-agent')

type State = 'online' | 'offline'

export class NetworkAgent {
  state: State = 'online'

  constructor(public events: WebClientEvents) { }

  async authorize(password: string) {
    this._assertNetworkState()

    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    if (response.ok) {
      return true
    }

    this._onNetworkError(response.status)

    return false
  }

  async syncChanges(
    rev: number,
    records: IRecord[],
    attachments: IAttachment[],
    files: { [hash: string]: Blob },
  ): Promise<IChangesetResult> {
    this._assertNetworkState()

    const changeset: IChangeset = {
      baseRev: rev,
      records,
      attachments,
    }

    const data = new FormData()
    data.append('changeset', JSON.stringify(changeset))
    for (const [hash, blob] of Object.entries(files)) {
      data.append(hash, blob)
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

  _assertNetworkState() {
    if (this.state === 'offline') {
      throw new Error('Network is offline')
    }
  }

  _onNetworkError(status: number) {
    this.events.emit('network-error', status)
    log.warn(`network error, http status code ${status}`)
  }

  _onNetworkConnectionChange = () => {
    this.state = window.navigator.onLine ? 'online' : 'offline'
    this.events.emit('network-online', this.state === 'online')
    log.info(`network gone ${this.state}`)
  }

  isOnline() {
    return this.state === 'online'
  }

  start() {
    window.addEventListener('online', this._onNetworkConnectionChange)
    window.addEventListener('offline', this._onNetworkConnectionChange)
    log.debug('started')
  }

  stop() {
    window.removeEventListener('online', this._onNetworkConnectionChange)
    window.removeEventListener('offline', this._onNetworkConnectionChange)
    log.debug('stopped')
  }
}
