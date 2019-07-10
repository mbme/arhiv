import { createLogger } from '~/logger'
import { IMergeConflicts } from '~/isodb-core/types'
import { isEmptyChangeset } from '~/isodb-core/utils'
import { IsodbReplica } from '../replica'
import { LockAgent } from './lock-agent'
import { NetworkAgent } from './network-agent'
import { AuthAgent } from './auth-agent'

const log = createLogger('isodb-web-client:sync-agent')

type AgentState = 'sync' | 'merge' | 'synced' | 'not-synced'

// TODO logs
// TODO circuit breaker
export class SyncAgent {
  private _syncIntervalId: number | undefined

  _state: AgentState = 'not-synced' // FIXME use this

  constructor(
    private replica: IsodbReplica,
    private lockAgent: LockAgent,
    private networkAgent: NetworkAgent,
    private authAgent: AuthAgent,
  ) { }

  private _merge = async (conflicts: IMergeConflicts) => {
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

  private _sync = async () => {
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
      this._state = 'sync'

      this.lockAgent.lockDB()

      let tries = 0
      while (true) {
        log.debug(`sync: trial #${tries + 1}`)
        const [changeset, localAttachments] = this.replica.getChangeset()

        if (isEmptyChangeset(changeset)) {
          log.info('sync: sending empty changeset')
        } else {
          // tslint:disable-next-line:max-line-length
          log.info(`sync: sending ${changeset.records.length} records, ${changeset.attachments.length} attachments (${Object.keys(localAttachments).length} BLOBs)`)
        }

        const result = await this.networkAgent.syncChanges(changeset, localAttachments)
        log.info(`sync: succes=${result.success}`)

        const needSync = await this.replica.applyChangesetResult(result, this._merge)

        if (!needSync) {
          log.debug('sync: ok')

          this._state = 'synced'

          return true
        }

        tries += 1
        if (tries > 2) {
          log.error('Failed to sync: exceeded max attemts')

          this._state = 'not-synced'

          return false
        }
      }
    } catch (e) {
      log.error('Failed to sync', e)

      this._state = 'not-synced'

      return false
    } finally {
      this.lockAgent.unlockDB()
    }
  }

  private _scheduleSync() {
    // tslint:disable-next-line:no-floating-promises
    this._sync()

    // schedule sync once per minute
    clearInterval(this._syncIntervalId)
    this._syncIntervalId = window.setInterval(this._sync, 60 * 1000)
  }

  _cancelSync() {
    clearInterval(this._syncIntervalId)
  }

  syncNow = () => {
    this._scheduleSync()
  }

  start() {
    this._scheduleSync()

    this.authAgent.events.on('authorized', this.syncNow)
  }

  stop() {
    this._cancelSync()

    this.authAgent.events.off('authorized', this.syncNow)
  }
}
