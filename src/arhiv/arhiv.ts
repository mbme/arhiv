import {
  Callbacks,
} from '~/utils'
import {
  ReplicaInMemStorage,
  IsodbReplica,
} from './isodb/replica'
import { LockManager } from './lock-manager'
import { NetworkManager } from './network-manager'
import { SyncManager } from './sync-manager'
import { ArhivReplica } from './types'
import {
  NotesRepository,
  TracksRepository,
  AttachmentsRepository,
} from './repositories'

export class Arhiv {
  private _callbacks = new Callbacks()

  private _net = new NetworkManager()
  private _locks = new LockManager()
  private _replica: ArhivReplica = new IsodbReplica(
    new ReplicaInMemStorage(),
    () => this.syncNow(),
  )
  private _sync = new SyncManager(this._replica, this._net, this._locks)

  readonly syncState$ = this._replica.syncState$
  readonly isAuthorized$ = this._net.isAuthorized$

  readonly attachments = new AttachmentsRepository(this._replica)
  readonly notes = new NotesRepository(this._replica, this._locks)
  readonly tracks = new TracksRepository(this._replica, this._locks)

  constructor() {
    this._callbacks.add(
      () => this._sync.stop(),
      () => this._locks.stop(),
      () => this._replica.stop(),
      () => this._net.stop(),
    )
  }

  syncNow() {
    this._sync.syncSignal.next()
  }

  authorize(password: string) {
    return this._net.authorize(password)
  }

  deauthorize() {
    this._net.deauthorize()
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
