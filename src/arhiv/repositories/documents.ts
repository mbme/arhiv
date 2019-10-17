import { Observable } from '~/utils/reactive'
import {
  ArhivReplica,
  Record,
} from '../types'
import { Document } from '../entities/document'
import { LockManager } from '../managers'

type IsDocumentOfType<T extends Record> = (x: any) => x is T

export abstract class DocumentsRepository<T extends Record> {
  constructor(
    protected _replica: ArhivReplica,
    protected _locks: LockManager,
    protected _isDocumentOfType: IsDocumentOfType<T>,
  ) { }

  protected _wrap = (document: T) => new Document(this._replica, this._locks, document)

  getDocuments$(): Observable<Array<Document<T>>> {
    return this._replica.getDocuments$()
      .map(documents => documents.filter(this._isDocumentOfType).map(this._wrap))
  }

  getDocument$(id: string): Observable<Document<T>> {
    return this._replica.getDocument$(id).map((document) => {
      if (this._isDocumentOfType(document)) {
        return this._wrap(document)
      }

      throw new Error(`document ${id} has wrong type`)
    })
  }
}
