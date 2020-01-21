import { createLogger } from '~/logger'
import {
  Callbacks,
} from '~/utils'
import {
  Observable,
  merge$,
  interval$,
  Signal,
  promise$,
} from '~/reactive'
import { ReplicaDB } from '../db'
import { NetworkManager } from './network-manager'
import { LockManager } from './lock-manager'

const log = createLogger('arhiv:sync-manager', 'magenta')

export class SyncManager {
  readonly syncSignal = new Signal()

  private _callbacks = new Callbacks()

  constructor(
    private _db: ReplicaDB,
    private _net: NetworkManager,
    private _locks: LockManager,
  ) {
    const mergeConflictsResolved$ = _db.syncState$.value$
      .buffer(2)
      .filter(syncStates => syncStates.length === 2 && syncStates[0].type === 'merge-conflicts')

    const gotAuthorized$ = _net.isAuthorized$.value$.filter(isAuthorized => isAuthorized)

    const gotLocalUpdate$ = _db.updateTime$.value$
      .filter(([, isLocal]) => isLocal)

    const syncCondtion$ = merge$<any>(
      // only leader should schedule sync
      _locks.acquireLeaderLock$().switchMap(() => merge$<any>(
        interval$(60 * 1000).tap(() => log.debug('interval trigger')),

        // got online
        _net.isOnline$.value$.filter(isOnline => isOnline)
          .tap(() => log.debug('network online trigger')),
      )),
      gotAuthorized$.tap(() => log.debug('auth trigger')),
      gotLocalUpdate$.tap(() => log.debug('local update trigger')),
      mergeConflictsResolved$.tap(() => log.debug('merge conflict resolved trigger')),
      this.syncSignal.signal$.tap(() => log.debug('signal trigger')),
    )
      .filter(() => this._isReadyToSync())
      .switchMap(() => this._startSync$())

    this._callbacks.add(
      syncCondtion$.subscribe({
        next: (success) => {
          log.info('synced -> ', success)
        },
      }),
    )
  }

  private _isReadyToSync() {
    return this._net.isOnline()
      && this._net.isAuthorized()
      && !this._locks.isDBLocked()
      && this._db.isReadyToSync()
  }

  private _startSync$() {
    return new Observable<boolean>(
      observer => this._locks.acquireDBLock$()
        .switchMap(() => promise$(this._db.sync(this._net.syncChanges)))
        .take(1)
        .subscribe({
          next: observer.next,
          error: (err) => {
            log.error('sync failed', err)
            observer.next(false)
          },
          complete: observer.complete,
        }),
    )
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
