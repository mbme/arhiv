import { Observable } from '~/reactive'
import {
  ArhivReplica,
  Record,
} from '../types'
import { Document } from '../entities/document'
import { LockManager } from '../managers'

export interface IDocumentType<T extends Record> {
  is(x: unknown): x is T
  create(id: string): T
}

export class DocumentsRepository<T extends Record> {
  constructor(
    private _replica: ArhivReplica,
    private _locks: LockManager,
    private _documentType: IDocumentType<T>,
  ) { }

  private _wrap = (document: T, isNew = false) => new Document<T>(this._replica, this._locks, document, isNew)

  async create(): Promise<Document<T>> {
    const id = await this._replica.getRandomId()

    return this._wrap(this._documentType.create(id), true)
  }

  getDocuments$(): Observable<Array<Document<T>>> {
    return this._replica.getDocuments$()
      .map(documents => {
        const result: Array<Document<T>> = []
        for (const document of documents) {
          if (this._documentType.is(document)) {
            result.push(this._wrap(document))
          }
        }

        return result
      })
  }

  getDocument$(id: string): Observable<Document<T>> {
    return this._replica.getDocument$(id).map((document) => {
      if (this._documentType.is(document)) {
        return this._wrap(document)
      }

      throw new Error(`document ${id} has wrong type`)
    })
  }
}
