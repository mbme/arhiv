import { Observable } from '~/reactive'
import { IDocument } from '~/arhiv/schema'
import { ReplicaDB } from '../db'
import { LockManager } from '../managers'
import { NoteManager } from './note-manager'

const RegisteredDocumentManagers = {
  'note': NoteManager,
}
type TDocumentTypes = keyof typeof RegisteredDocumentManagers
type TDocumentManagers = InstanceType<typeof RegisteredDocumentManagers[TDocumentTypes]>
type TDocumentManager<T extends TDocumentTypes> = InstanceType<typeof RegisteredDocumentManagers[T]>

export class DocumentsRepository {
  constructor(
    private _db: ReplicaDB,
    private _locks: LockManager,
  ) { }

  private _wrap(document: IDocument): TDocumentManagers {
    const Manager = (RegisteredDocumentManagers as any)[document.type]
    if (!Manager) {
      throw new Error(`Got unexpected document type ${document.type}`)
    }

    return new Manager(this._db, this._locks, document)
  }

  async create<T extends TDocumentTypes>(type: T): Promise<TDocumentManager<T>> {
    const id = await this._db.getRandomId()

    const Manager = RegisteredDocumentManagers[type] as any

    return new Manager(this._db, this._locks, id)
  }

  getDocuments$(): Observable<TDocumentManagers[]> {
    return this._db.getDocuments$()
      .map(documents => documents.map(document => this._wrap(document)))
  }

  getDocument$(id: string): Observable<TDocumentManagers> {
    return this._db.getDocument$(id).map((document) => this._wrap(document))
  }
}
