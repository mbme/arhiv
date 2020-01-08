import { createLogger } from '~/logger'
import {
  selectLinks,
  parseMarkup,
} from '~/markup-parser'
import {
  IDocument,
} from '../../schema'
import {
  ReplicaDB,
} from '../db'
import { LockManager } from '../managers'

const log = createLogger('document')

// Active Record
export abstract class DocumentManager<P extends object> {
  constructor(
    private _db: ReplicaDB,
    private _locks: LockManager,
    public readonly document: IDocument,
    private _isNew: boolean,
  ) { }

  get id() {
    return this.document.id
  }

  private async _extractRefs(value: string) {
    const attachmentRefs: string[] = []

    const markup = parseMarkup(value)

    for (const link of selectLinks(markup)) {
      const id = link.link

      if (await this._db.getAttachment(id)) {
        attachmentRefs.push(id)
      } else {
        log.warn(`document ${this.id} references unknown entity ${id}`)
      }
    }

    return attachmentRefs
  }

  async patch(patch: Partial<P>) {
    const refSources = Object.values(patch)
      .filter(value => value instanceof MarkupString)

    // FIXME extract doc refs
    const attachmentRefs = new Set((await Promise.all(this._getMarkupStrings().map(this._extractRefs))).flat())

    const attachmentRefs = refSources.length
      ? await this._extractRefs(refSources.join(''))
      : this.document.attachmentRefs

    await this._db.saveDocument({
      ...this.document,
      ...patch,
      _attachmentRefs: attachmentRefs,
    })
    this._isNew = false
  }

  async delete() {
    await this._db.saveDocument({
      ...this.document,
      _deleted: true,
    })
  }

  isNew() {
    return this._isNew
  }

  isLocked$() {
    return this._locks.isDocumentLocked$(this.id)
  }

  acquireLock$() {
    return this._locks.acquireDocumentLock$(this.id)
  }
}
