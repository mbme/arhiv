import LockManager from './lock-manager'
import { IPatchResponse, ChangedRecord, Record } from '../core/types'
import ReplicaDB from '../core/replica'
import PubSub from '../../utils/pubsub'

interface IEvents {
  'unauthorized': undefined
  'network-error': number
}

interface ISyncState {
  state: 'sync'
}

interface IMergeState {
  state: 'merge'
}

interface ISyncedState {
  state: 'synced'
}

interface INotSyncedState {
  state: 'not-synced'
}

type AgentState = ISyncState | IMergeState | ISyncedState | INotSyncedState

// TODO logs
// TODO listen to network availability
// TODO circuit breaker
// TODO run isodb/web-client in a Shared Worker
export default class SyncAgent {
  events: PubSub<IEvents> = new PubSub()
  _syncIntervalId: number | undefined

  _state: AgentState = { state: 'not-synced' }

  constructor(
    public replica: ReplicaDB,
    public lockManager: LockManager
  ) { }

  _merge = async (_base: Record, _newBase: Record, local: ChangedRecord) => {
    // FIXME implement merge
    return local
  }

  async _pushChanges(
    rev: number,
    records: ChangedRecord[],
    attachments: { [hash: string]: Blob }
  ): Promise<IPatchResponse> {
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
      if (response.status === 403) {
        this.events.emit('unauthorized', undefined)
      } else {
        this.events.emit('network-error', response.status)
      }
      throw new Error(`Server responded with code ${response.status}`)
    }

    return response.json()
  }

  async _sync() {
    if (!this.lockManager.lockDB()) return false

    try {
      let tries = 0
      while (true) {
        const result = await this._pushChanges(
          this.replica.getRev(),
          this.replica.storage.getLocalRecords(),
          this.replica.storage.getLocalAttachments()
        )

        await this.replica.applyPatch(result, this._merge)

        if (result.applied) {
          return true
        }

        tries += 1
        if (tries > 2) {
          throw new Error('Failed to sync, try again later')
        }
      }
    } finally {
      this.lockManager.unlockDB()
    }
  }

  start() {
    // tslint:disable-next-line:no-floating-promises
    this._sync()

    // schedule sync once per minute
    this._syncIntervalId = window.setInterval(this._sync, 60 * 1000)
  }

  stop() {
    clearInterval(this._syncIntervalId)
  }
}
