import {
  nowS,
  Deferred,
} from '~/utils'
import { IDocument } from '../../types'

export class DocumentConflict<T extends IDocument> {
  private _deffered = new Deferred<T>()
  private _final?: T

  constructor(
    public readonly base: T,
    public readonly remote: T,
    public readonly local: T,
  ) { }

  get promise() {
    return this._deffered.promise
  }

  private _assertNotResolved() {
    if (this._final) {
      throw new Error('Conflict has already been resolved')
    }
  }

  isResolved() {
    return !!this._final
  }

  resolve(final: T) {
    this._assertNotResolved()

    this._final = {
      ...final,
      _rev: this.remote._rev,
      _updatedTs: nowS(),
    }
    this._deffered.resolve(this._final)
  }

  useLocal() {
    this._assertNotResolved()

    this.resolve(this.local)
  }

  useRemote() {
    this._assertNotResolved()

    this.resolve(this.remote)
  }
}

export class MergeConflicts<T extends IDocument> {
  public readonly promise: Promise<T[]>

  constructor(
    public readonly conflicts: Array<DocumentConflict<T>>,
    onResolved: (documents: T[]) => Promise<void>,
  ) {
    this.promise = Promise.all(conflicts.map(conflict => conflict.promise)).then(async (documents) => {
      await onResolved(documents)

      return documents
    })
  }
}
