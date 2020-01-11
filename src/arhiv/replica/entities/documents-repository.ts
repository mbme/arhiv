import { Observable } from '~/turbo'
import { IDocument } from '~/arhiv/types'
import { ReplicaDB } from '../db'
import { Document } from './document'
import { DocumentNote } from './document-note'

interface IDocumentClass<P extends Document> {
  type: string
  new(db: ReplicaDB, document: IDocument<string, any>): P
  create(db: ReplicaDB): Promise<P>
}

const DOCUMENT_CLASSES: Array<IDocumentClass<any>> = [
  DocumentNote,
]

export class DocumentsRepository {
  constructor(
    private _db: ReplicaDB,
  ) { }

  async create<P extends Document>(DocumentType: IDocumentClass<P>) {
    return DocumentType.create(this._db)
  }

  private _wrap = (document: IDocument): Document => {
    for (const DocumentClass of DOCUMENT_CLASSES) {
      if (DocumentClass.type === document.type) {
        return new DocumentClass(this._db, document)
      }
    }

    throw new Error(`Got unknown document type ${document.type}`)
  }

  // FIXME this should accept IQuery with filter, limit etc
  getDocuments$(): Observable<Document[]> {
    return this._db.getDocuments$()
      .map(documents => documents.map(this._wrap))
  }

  getDocument$(id: string): Observable<Document> {
    return this._db.getDocument$(id, true).map(this._wrap)
  }
}
