import {
  Deferred,
  dateNow,
} from '~/utils'
import {
  ArhivDocument,
} from '../../types'

export class DocumentConflict {
  private _deffered = new Deferred<ArhivDocument>()
  private _final?: ArhivDocument

  constructor(
    public readonly base: ArhivDocument,
    public readonly remote: ArhivDocument,
    public readonly local: ArhivDocument,
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

  resolve(final: ArhivDocument) {
    this._assertNotResolved()

    this._final = {
      ...final,
      _rev: this.remote._rev,
      _updatedAt: dateNow(),
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

export class MergeConflicts {
  public readonly promise: Promise<ArhivDocument[]>

  constructor(
    public readonly conflicts: DocumentConflict[],
    onResolved: (documents: ArhivDocument[]) => Promise<void>,
  ) {
    this.promise = Promise.all(conflicts.map(conflict => conflict.promise)).then(async (documents) => {
      await onResolved(documents)

      return documents
    })
  }
}
