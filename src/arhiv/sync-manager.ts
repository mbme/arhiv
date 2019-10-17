import {
  createLogger,
  Callbacks,
} from '~/utils'
import {
  Observable,
  merge$,
  interval$,
  Signal,
  promise$,
} from '~/utils/reactive'
import { ArhivReplica } from './types'
import { NetworkManager } from './network-manager'
import { LockManager } from './lock-manager'

const log = createLogger('arhiv:sync-manager')

export class SyncManager {
  readonly syncSignal = new Signal()

  private _callbacks = new Callbacks()

  constructor(
    private _replica: ArhivReplica,
    private _net: NetworkManager,
    private _locks: LockManager,
  ) {
    const mergeConflictsResolved$ = _replica.syncState$.value$
      .buffer(2)
      .filter(syncStates => syncStates.length === 2 && syncStates[0].type === 'merge-conflicts')

    const gotAuthorized$ = _net.isAuthorized$.value$.filter(isAuthorized => isAuthorized)

    const gotOnline$ = _net.isOnline$.value$.filter(isOnline => isOnline)

    const syncCondtion$ = merge$<any>(
      interval$(60 * 1000),
      gotAuthorized$,
      gotOnline$,
      mergeConflictsResolved$,
      this.syncSignal.signal$,
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
      && this._replica.isReadyToSync()
  }

  private _startSync$() {
    return new Observable<boolean>((observer) => {
      this._locks.acquireDBLock$()
        .switchMap(() => promise$(this._replica.sync(this._net.syncChanges)))
        .take(1)
        .subscribe({
          next: observer.next,
          error: (err) => {
            log.error('sync failed', err)
            observer.next(false)
          },
          complete: observer.complete,
        })
    })
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
