import { nowS } from '~/utils'
import { IDocument } from '../types'

class DocumentConflict<T extends IDocument> {
  final?: T

  constructor(
    public base: T,
    public remote: T,
    public local: T,
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
  conflicts: Array<DocumentConflict<T>> = []

  constructor(private onResolved: (documents: T[]) => void) { }

  addConflict(base: T, remote: T, local: T) {
    this.conflicts.push(new DocumentConflict(base, remote, local, this._onResolved))
  }

  private _onResolved = () => {
    const pendingConflicts = this.conflicts.filter(conflict => !conflict.isResolved())
    if (pendingConflicts.length) {
      return
    }

    this.onResolved(this.conflicts.map(conflict => conflict.final!))
  }
}
