import {
  Deferred,
} from '~/utils'
import { dateNow } from '~/chrono'
import { IDocument } from '~/arhiv/types'

export class DocumentConflict {
  private _deffered = new Deferred<IDocument>()
  private _final?: IDocument

  constructor(
    public readonly base: IDocument,
    public readonly remote: IDocument,
    public readonly local: IDocument,
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

  resolve(final: IDocument) {
    this._assertNotResolved()

    this._final = {
      ...final,
      rev: this.remote.rev,
      updatedAt: dateNow(),
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
  public readonly promise: Promise<IDocument[]>

  constructor(
    public readonly conflicts: DocumentConflict[],
    onResolved: (documents: IDocument[]) => Promise<void>,
  ) {
    this.promise = Promise.all(conflicts.map(conflict => conflict.promise)).then(async (documents) => {
      await onResolved(documents)

      return documents
    })
  }
}
