import {
  Callbacks,
} from '~/utils'
import {
  TIDBStorage,
  IsodbReplica,
} from './isodb/replica'
import {
  LockManager,
  NetworkManager,
  SyncManager,
} from './managers'
import {
  ArhivReplica,
  Record,
  DocumentType,
  INote,
  ITrack,
} from './types'
import {
  DocumentsRepository,
  AttachmentsRepository,
  IDocumentType,
  Document,
} from './entities'
import {
  createDocument,
} from './isodb/utils'

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

export class Arhiv {
  private _callbacks = new Callbacks()

  private _net = new NetworkManager()
  private _locks = new LockManager()
  private _replica: ArhivReplica = new IsodbReplica(this._db)
  private _sync = new SyncManager(this._replica, this._net, this._locks)

  readonly syncState$ = this._replica.syncState$
  readonly isAuthorized$ = this._net.isAuthorized$

  readonly attachments = new AttachmentsRepository(this._replica)
  readonly notes = new DocumentsRepository(this._replica, this._locks, NoteType)
  readonly tracks = new DocumentsRepository(this._replica, this._locks, TrackType)

  private constructor(private _db: TIDBStorage<Record>) {
    this._callbacks.add(
      () => this._sync.stop(),
      () => this._locks.stop(),
      () => this._replica.stop(),
      () => this._net.stop(),
    )
  }

  private async _start() {
    await this._replica.compact()
  }

  static async create() {
    const db = await TIDBStorage.open<Record>()

    const arhiv = new Arhiv(db)
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
