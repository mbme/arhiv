import { ReactiveValue } from '~/utils/reactive'
import {
  ArhivReplica,
  Record,
} from '../types'
import { ArhivRecord } from './record'

type IsDocumentOfType<T extends Record> = (x: any) => x is T

export abstract class BaseRepository<T extends Record> {
  constructor(
    protected _replica: ArhivReplica,
    protected _isDocumentOfType: IsDocumentOfType<T>,
  ) { }

  protected _wrap = (document: T) => new ArhivRecord(this._replica, document)

  getDocuments(): ReactiveValue<Array<ArhivRecord<T>>> {
    return this._replica.getDocuments()
      .map(documents => documents.filter(this._isDocumentOfType).map(this._wrap))
  }

  getDocument(id: string): ReactiveValue<ArhivRecord<T> | undefined> {
    return this._replica.getDocument(id).map((document) => {
      if (this._isDocumentOfType(document)) {
        return this._wrap(document)
      }

      return undefined
    })
  }
}
