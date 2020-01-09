import { Observable } from '~/reactive'
import { createDocument } from '~/arhiv/utils'
import { ReplicaDB } from '../db'
import { DocumentManager } from './document-manager'
import { LockManager } from '../managers'

export class DocumentsRepository {
  constructor(
    private _db: ReplicaDB,
    private _locks: LockManager,
  ) { }

  async create<T extends string, P extends object>(type: T, initialProps: P): Promise<DocumentManager<T, P>> {
    const id = await this._db.getRandomId()

    const document = createDocument(id, type, initialProps)

    return new DocumentManager(this._db, this._locks, document, true)
  }

  getDocuments$(): Observable<DocumentManager[]> {
    return this._db.getDocuments$()
      .map(documents => documents.map(document => new DocumentManager(this._db, this._locks, document)))
  }

  getDocument$(id: string): Observable<DocumentManager> {
    return this._db.getDocument$(id).map(document => new DocumentManager(this._db, this._locks, document))
  }
}
