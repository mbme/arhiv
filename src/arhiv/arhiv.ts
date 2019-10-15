import {
  Callbacks,
  createLogger,
} from '~/utils'
import {
  Signal,
  interval$,
  merge$,
  promise$,
  Observable,
} from '~/utils/reactive'
import {
  ReplicaInMemStorage,
  IsodbReplica,
} from './isodb/replica'
import { LockManager } from './lock-manager'
import { NetworkManager } from './network-manager'
import { ArhivReplica } from './types'
import {
  NotesRepository,
  TracksRepository,
  AttachmentsRepository,
} from './repositories'

const log = createLogger('arhiv')

export class Arhiv {
  readonly net = new NetworkManager()

  private _locks = new LockManager()
  private _replica: ArhivReplica = new IsodbReplica(new ReplicaInMemStorage())
  private _callbacks = new Callbacks()
  private _syncSignal = new Signal()

  readonly syncState$ = this._replica.syncState$

  readonly attachments = new AttachmentsRepository(this._replica)
  readonly notes = new NotesRepository(this._replica, this._locks)
  readonly tracks = new TracksRepository(this._replica, this._locks)

  constructor() {
    const mergeConflictsResolved$ = this._replica.syncState$.value$
      .buffer(2)
      .filter(syncStates => syncStates.length === 2 && syncStates[0].type === 'merge-conflicts')
    const gotAuthorized$ = this.net.authorized$.value$.filter(isAuthorized => isAuthorized)
    const gotOnline$ = this.net.isOnline$.value$.filter(isOnline => isOnline)

    const syncCondtion$ = merge$<any>(
      interval$(60 * 1000),
      gotAuthorized$,
      gotOnline$,
      mergeConflictsResolved$,
      this._syncSignal.signal$,
    )
      .filter(() => this.net.isOnline()
        && this.net.isAuthorized()
        && this._locks.isDBLockable()
        && this._replica.isReadyToSync())
      .switchMap(
        () => new Observable<boolean>((observer) => {
          this._locks.acquireDBLock$()
            .switchMap(() => promise$(this._replica.sync(this.net.syncChanges)))
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
      () => this._locks.stop(),
      () => this._replica.stop(),
      () => this.net.stop(),
      syncCondtion$.subscribe({
        next: (success) => {
          log.info('synced -> ', success)
        },
      }),
    )
  }

  readonly syncNow = () => {
    this._syncSignal.next()
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
