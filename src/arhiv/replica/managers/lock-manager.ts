import { createLogger } from '~/logger'
import {
  Callbacks,
  Dict,
} from '~/utils'
import {
  Observable,
} from '~/reactive'
import {
  WebLocks,
  getTabId,
} from '~/web-utils'

const log = createLogger('lock-manager')

const DB_LOCK_PROP = 'db-locked'
const LEADER_LOCK_PROP = 'leader-lock'

function getDocumentLocks(locks: Dict): Dict {
  const result = {
    ...locks,
  }

  // tslint:disable-next-line:no-dynamic-delete
  delete result[DB_LOCK_PROP]
  // tslint:disable-next-line:no-dynamic-delete
  delete result[LEADER_LOCK_PROP]

  return result
}

export class LockManager {
  private _locks = new WebLocks(getTabId('-v-tab-id').toString(), '-v-locks')

  private _callbacks = new Callbacks()

  constructor() {
    this._callbacks.add(
      this._locks.state.value$.subscribe({
        next(currentState) {
          const dbLockOwner = currentState[DB_LOCK_PROP]
          const leaderLockOwner = currentState[LEADER_LOCK_PROP]

          const logParts = [
            leaderLockOwner ? `leader: ${leaderLockOwner}` : 'no leader',
            dbLockOwner ? `db locked by ${dbLockOwner}` : '',
          ]

          for (const [lockId, tabId] of Object.entries(getDocumentLocks(currentState))) {
            logParts.push(`document ${lockId} locked by ${tabId}`)
          }

          log.debug('lock state ->', logParts.join(', '))
        },
      }),
    )
  }

  isDocumentLocked$(id: string): Observable<boolean> {
    return this._locks.state.value$.map(locks => Boolean(locks[id] || locks[DB_LOCK_PROP]))
  }

  acquireDocumentLock$(id: string) {
    return this.isDocumentLocked$(id)
      .filter(isLocked => !isLocked)
      .take(1)
      .switchMap(() => this._locks.acquireLock$(id))
  }

  isDBLocked(): boolean {
    return !!this._locks.state.value[DB_LOCK_PROP]
  }

  acquireDBLock$() {
    return this._locks.state.value$.map((state) => {
      if (state[DB_LOCK_PROP]) {
        return false
      }

      return Object.keys(getDocumentLocks(state)).length === 0
    })
      .filter(isFree => isFree)
      .take(1)
      .switchMap(() => this._locks.acquireLock$(DB_LOCK_PROP))
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
