import { Observable } from '~/reactive'
import { ReplicaDB } from '../db'
import {
  ArhivDocumentType,
  ArhivDocument,
} from '../../types'
import {
  createDocument,
} from '../../utils'
import { Document } from './document'
import { LockManager } from '../managers'

export class DocumentsRepository {
  constructor(
    private _db: ReplicaDB,
    private _locks: LockManager,
  ) { }

  private _wrap<T extends ArhivDocument>(document: T, isNew = false) {
    return new Document<T>(this._db, this._locks, document, isNew)
  }

  async create<T extends ArhivDocumentType>(type: T) {
    const id = await this._db.getRandomId()
    const document = createDocument(id, type)

    return this._wrap(document, true)
  }

  getDocuments$(): Observable<Document[]> {
    return this._db.getDocuments$()
      .map(documents => documents.map(document => this._wrap(document)))
  }

  getDocument$(id: string): Observable<Document> {
    return this._db.getDocument$(id).map((document) => this._wrap(document))
  }
}
