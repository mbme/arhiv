import { IDocument } from '~/arhiv/types'
import { createDocument } from '~/arhiv/utils'
import { Observable } from '~/reactive'
import { ReplicaDB } from '../db'
import { Document } from './document'

interface IQuery {
  filter?: (document: Document) => boolean
  includeDeleted?: boolean
}

export class DocumentsRepository {
  constructor(
    private _db: ReplicaDB,
  ) { }

  private _wrap = (document: IDocument): Document => new Document(this._db, document)

  async create<P extends object>(type: string, props: P) {
    const id = await this._db.getRandomId()

    const document = createDocument(id, type, props)

    return new Document(this._db, document)
  }

  getDocuments$(query: IQuery): Observable<Document[]> {
    return this._db.getDocuments$()
      .map(documents => (
        documents
          .map(this._wrap)
          .filter((document) => {
            if (!query.includeDeleted && document.deleted) {
              return false
            }

            if (!query.filter) {
              return true
            }

            return query.filter(document)
          })
      ))
  }

  getDocument$(id: string): Observable<Document> {
    return this._db.getDocument$(id, true).map(this._wrap)
  }
}
