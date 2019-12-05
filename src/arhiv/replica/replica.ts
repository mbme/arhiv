import {
  Callbacks,
} from '~/utils'
import {
  getTabId,
} from '~/web-utils'
import {
  TIDBStorage,
  ArhivDB,
} from './db'
import {
  LockManager,
  NetworkManager,
  SyncManager,
} from './managers'
import {
  Record,
  DocumentType,
  INote,
  ITrack,
} from '../types'
import {
  createDocument,
} from '../utils'
import {
  DocumentsRepository,
  AttachmentsRepository,
  IDocumentType,
  Document,
} from './entities'

const NoteType: IDocumentType<INote> = {
  is(x: any): x is INote {
    // tslint:disable-next-line:no-unsafe-any
    return x && x._type === DocumentType.Note
  },
  create(id: string): INote {
    return ({
      ...createDocument(id, DocumentType.Note),
      name: '',
      data: '',
    })
  },
}
export type NoteDocument = Document<INote>

const TrackType: IDocumentType<ITrack> = {
  is(x: any): x is ITrack {
    // tslint:disable-next-line:no-unsafe-any
    return x && x._type === DocumentType.Track
  },

  create(id: string): ITrack {
    return ({
      ...createDocument(id, DocumentType.Track),
      title: '',
      artist: '',
    })
  },
}
export type TrackDocument = Document<ITrack>

export class ArhivReplica {
  private _callbacks = new Callbacks()

  readonly tabId = getTabId('-v-tab-id').toString()

  private _net = new NetworkManager()
  private _locks = new LockManager(this.tabId)
  private _db = new ArhivDB(this._storage)
  private _sync = new SyncManager(this._db, this._net, this._locks)

  readonly syncState$ = this._db.syncState$
  readonly isAuthorized$ = this._net.isAuthorized$

  readonly attachments = new AttachmentsRepository(this._db)
  readonly notes = new DocumentsRepository(this._db, this._locks, NoteType)
  readonly tracks = new DocumentsRepository(this._db, this._locks, TrackType)

  private constructor(private _storage: TIDBStorage<Record>) {
    this._callbacks.add(
      () => this._sync.stop(),
      () => this._locks.stop(),
      () => this._db.stop(),
      () => this._net.stop(),
    )
  }

  private async _start() {
    await this._db.compact()
  }

  static async create() {
    const db = await TIDBStorage.open<Record>()

    const arhiv = new ArhivReplica(db)
    await arhiv._start()

    return arhiv
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
