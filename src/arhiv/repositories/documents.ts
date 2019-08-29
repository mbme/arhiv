import { ReactiveValue } from '~/utils'
import {
  ArhivReplica,
  Record,
} from '../types'
import { Document } from '../entities/document'

type IsDocumentOfType<T extends Record> = (x: any) => x is T

export abstract class DocumentsRepository<T extends Record> {
  constructor(
    protected _replica: ArhivReplica,
    protected _isDocumentOfType: IsDocumentOfType<T>,
  ) { }

  protected _wrap = (document: T) => new Document(this._replica, document)

  getDocuments(): ReactiveValue<Array<Document<T>>> {
    return this._replica.getDocuments()
      .map(documents => documents.filter(this._isDocumentOfType).map(this._wrap))
  }

  getDocument(id: string): ReactiveValue<Document<T> | undefined> {
    return this._replica.getDocument(id).map((document) => {
      if (this._isDocumentOfType(document)) {
        return this._wrap(document)
      }

      return undefined
    })
  }
}
