import LockAgent from './lock-agent'
import {
  IMergeConflict,
} from '../core/types'
import ReplicaDB from '../core/replica'
import { createLogger } from '../../logger'
import NetworkAgent from './network-agent'

const log = createLogger('isodb-web-client:sync-agent')

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
// TODO circuit breaker
export default class SyncAgent {
  _syncIntervalId: number | undefined

  _state: AgentState = { state: 'not-synced' }

  constructor(
    public replica: ReplicaDB,
    public lockAgent: LockAgent,
    public networkAgent: NetworkAgent
  ) { }

  _merge = async (conflicts: IMergeConflict[]) => {
    log.warn(`${conflicts.length} merge conflicts`)
    // FIXME implement merge
    return conflicts.map(conflict => conflict.local)
  }

  async _sync() {
    if (!this.lockAgent.lockDB()) return false

    try {
      let tries = 0
      while (true) {
        const result = await this.networkAgent.syncChanges(
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
      this.lockAgent.unlockDB()
    }
  }

  _scheduleSync() {
    // tslint:disable-next-line:no-floating-promises
    this._sync()

    // schedule sync once per minute
    clearInterval(this._syncIntervalId)
    this._syncIntervalId = window.setInterval(this._sync, 60 * 1000)
  }

  _cancelSync() {
    clearInterval(this._syncIntervalId)
  }

  start() {
    this._scheduleSync()
  }

  stop() {
    this._cancelSync()
  }
}
