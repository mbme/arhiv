import { Callbacks } from '~/utils'
import {
  Signal,
  interval$,
  merge$,
} from '~/utils/reactive'
import {
  ReplicaInMemStorage,
  ReplicaManager,
} from '~/isodb/replica'
import { LockManager } from './lock-manager'
import { NetworkManager } from './network-manager'
import { ArhivReplica } from './types'
import {
  NotesRepository,
  TracksRepository,
  AttachmentsRepository,
} from './repositories'

export class Arhiv {
  readonly net = new NetworkManager()

  private _locks = new LockManager()
  private _replica: ArhivReplica = new ReplicaManager(new ReplicaInMemStorage())
  private _callbacks = new Callbacks()
  private _syncSignal = new Signal()

  readonly attachments = new AttachmentsRepository(this._replica)
  readonly notes = new NotesRepository(this._replica, this._locks)
  readonly tracks = new TracksRepository(this._replica, this._locks)

  constructor() {
    const syncCondtion$ = merge$<any>(
      interval$(60 * 1000),
      this.net.authorized$.value$.filter(isAuthorized => isAuthorized),
      this._replica.syncState$.value$.filter(syncState => syncState === 'merge-conflicts-resolved'),
      this._syncSignal.signal$,
    )

    this._callbacks.add(
      () => this._locks.stop(),
      () => this._replica.stop(),
      () => this.net.stop(),
      syncCondtion$.subscribe({
        next: () => {
          // tslint:disable-next-line:no-floating-promises
          // FIXME acquire db lock
          this._replica.sync(this.net.syncChanges)
        },
      }),
    )
  }

  syncNow = () => {
    this._syncSignal.next()
  }

  stop() {
    this._callbacks.runAll()
  }
}
