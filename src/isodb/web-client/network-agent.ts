import {
  IPatchResponse,
  ChangedRecord,
} from '../core/types'
import { WebClientEvents } from './events'

type State = 'online' | 'offline'

export default class NetworkAgent {
  state: State = 'online'

  constructor(public events: WebClientEvents) { }

  _assertNetworkState() {
    if (this.state === 'offline') {
      throw new Error('Network is offline')
    }
  }

  async authorize(password: string) {
    this._assertNetworkState()

    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    if (response.status === 200) {
      return true
    }

    if (response.status !== 401) {
      this.events.emit('network-error', response.status)
    }

    return false
  }

  deauthorize() {
    document.cookie = 'token=0; path=/'
  }

  async syncChanges(
    rev: number,
    records: ChangedRecord[],
    attachments: { [hash: string]: Blob }
  ): Promise<IPatchResponse> {
    this._assertNetworkState()

    const data = new FormData()
    data.append('rev', rev.toString())
    data.append('records', JSON.stringify(records))
    for (const [hash, blob] of Object.entries(attachments)) {
      data.append(hash, blob)
    }

    const response = await fetch('/api/changes', {
      method: 'post',
      credentials: 'include',
      body: data,
    })

    if (!response.ok) {
      this.events.emit('network-error', response.status)
      throw new Error(`Server responded with code ${response.status}`)
    }

    return response.json()
  }

  _onNetworkConnectionChange = () => {
    this.state = window.navigator.onLine ? 'online' : 'offline'
    this.events.emit('network-online', this.state === 'online')
  }

  start() {
    window.addEventListener('online', this._onNetworkConnectionChange)
    window.addEventListener('offline', this._onNetworkConnectionChange)
  }

  stop() {
    window.removeEventListener('online', this._onNetworkConnectionChange)
    window.removeEventListener('offline', this._onNetworkConnectionChange)
  }
}
