import { createLogger } from '~/logger'
import {
  IMergeConflicts,
} from '~/isodb-core/types'
import { IsodbReplica } from '../replica'
import { LockAgent } from './lock-agent'
import { NetworkAgent } from './network-agent'
import { AuthAgent } from './auth-agent'

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
export class SyncAgent {
  _syncIntervalId: number | undefined

  _state: AgentState = { state: 'not-synced' }

  constructor(
    public replica: IsodbReplica,
    public lockAgent: LockAgent,
    public networkAgent: NetworkAgent,
    public authAgent: AuthAgent,
  ) { }

  _merge = async (conflicts: IMergeConflicts) => {
    // tslint:disable-next-line:max-line-length
    log.warn(`${conflicts.records.length} record merge conflicts, ${conflicts.attachments.length} attachment merge conflicts`)

    // FIXME implement merge
    const records = conflicts.records.map(conflict => conflict.local)
    const attachments = conflicts.attachments.map(conflict => conflict.local)

    return {
      records,
      attachments,
    }
  }

  _sync = async () => {
    if (!this.lockAgent.isFree()) {
      log.debug('Skipping sync: lock is not free')

      return false
    }

    if (!this.networkAgent.isOnline()) {
      log.debug('Skipping sync: network is offline')

      return false
    }

    if (!this.authAgent.isAuthorized()) {
      log.debug('Skipping sync: not authorized')

      return false
    }

    try {
      this.lockAgent.lockDB()

      let tries = 0
      while (true) {
        log.debug(`sync: trial #${tries + 1}`)
        const result = await this.networkAgent.syncChanges(
          this.replica.getRev(),
          this.replica._storage.getLocalRecords(),
          this.replica._storage.getLocalAttachments(),
          this.replica._storage.getLocalAttachmentsData(),
        )

        await this.replica.applyChangesetResult(result, this._merge)

        if (result.success) {
          log.debug('sync: ok')

          return true
        }

        tries += 1
        if (tries > 2) {
          log.error('Failed to sync: exceeded max attemts')

          return false
        }
      }
    } catch (e) {
      log.error('Failed to sync', e)

      return false
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

  syncNow() {
    this._scheduleSync()
  }

  start() {
    this._scheduleSync()
  }

  stop() {
    this._cancelSync()
  }
}
