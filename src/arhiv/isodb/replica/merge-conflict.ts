import { nowS } from '~/utils'
import { IDocument } from '../types'

class DocumentConflict<T extends IDocument> {
  final?: T

  constructor(
    public readonly base: T,
    public readonly remote: T,
    public readonly local: T,
    private _onResolved: () => void,
  ) { }

  isResolved() {
    return !!this.final
  }

  resolve(final: T) {
    this.final = {
      ...final,
      _rev: this.remote._rev,
      _updatedTs: nowS(),
    }
    this._onResolved()
  }

  useLocal() {
    this.resolve(this.local)
  }

  useRemote() {
    this.resolve(this.remote)
  }
}

export class MergeConflicts<T extends IDocument> {
  readonly conflicts: Array<DocumentConflict<T>> = []

  constructor(
    private _onConflictsResolved: (documents: readonly T[]) => void,
  ) { }

  addConflict(base: T, remote: T, local: T) {
    this.conflicts.push(new DocumentConflict(base, remote, local, this._onResolved))
  }

  private _onResolved = () => {
    const pendingConflicts = this.conflicts.filter(conflict => !conflict.isResolved())
    if (pendingConflicts.length) {
      return
    }

    this._onConflictsResolved(this.conflicts.map(conflict => conflict.final!))
  }
}
