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
    replica: ArhivReplica,
    net: NetworkManager,
    locks: LockManager,
  ) {
    const mergeConflictsResolved$ = replica.syncState$.value$
      .buffer(2)
      .filter(syncStates => syncStates.length === 2 && syncStates[0].type === 'merge-conflicts')

    const gotAuthorized$ = net.isAuthorized$.value$.filter(isAuthorized => isAuthorized)

    const gotOnline$ = net.isOnline$.value$.filter(isOnline => isOnline)

    const syncCondtion$ = merge$<any>(
      interval$(60 * 1000),
      gotAuthorized$,
      gotOnline$,
      mergeConflictsResolved$,
      this.syncSignal.signal$,
    )
      .filter(() => net.isOnline()
        && net.isAuthorized()
        && !locks.isDBLocked()
        && replica.isReadyToSync())
      .switchMap(
        () => new Observable<boolean>((observer) => {
          locks.acquireDBLock$()
            .switchMap(() => promise$(replica.sync(net.syncChanges)))
            .take(1)
            .subscribe({
              next: observer.next,
              error: (err) => {
                log.error('sync failed', err)
                observer.next(false)
              },
              complete: observer.complete,
            })
        }),
      )

    this._callbacks.add(
      syncCondtion$.subscribe({
        next: (success) => {
          log.info('synced -> ', success)
        },
      }),
    )
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
