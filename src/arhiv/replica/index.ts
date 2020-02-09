import {
  Callbacks,
} from '~/utils'
import {
  getTabId,
} from '~/web-utils'
import {
  TIDBStorage,
  ReplicaDB,
} from './db'
import {
  LockManager,
  NetworkManager,
  SyncManager,
} from './managers'
import {
  DocumentsRepository,
  AttachmentsRepository,
} from './entities'

export {
  Attachment,
  Document,
} from './entities'

export class ArhivReplica {
  private _callbacks = new Callbacks()

  readonly tabId = getTabId('-v-tab-id').toString()

  private _net = new NetworkManager()
  private _locks = new LockManager(this.tabId)
  private _db = new ReplicaDB(this._storage)
  private _sync = new SyncManager(this._db, this._net, this._locks)

  readonly syncState$ = this._db.syncState$

  readonly attachments = new AttachmentsRepository(this._db)
  readonly documents = new DocumentsRepository(this._db)

  private constructor(private _storage: TIDBStorage) {
    this._callbacks.add(
      () => this._sync.stop(),
      () => this._locks.stop(),
      () => this._db.stop(),
      () => this._net.stop(),
    )
  }

  static async create() {
    const db = await TIDBStorage.open()

    const arhiv = new ArhivReplica(db)
    await db.compact()

    return arhiv
  }

  syncNow() {
    this._sync.syncSignal.next()
  }

  isDocumentLocked$(id: string) {
    return this._locks.isDocumentLocked$(id)
  }

  acquireDocumentLock$(id: string) {
    return this._locks.acquireDocumentLock$(id).map(() => true)
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
