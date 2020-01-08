import { createLogger } from '~/logger'
import {
  selectLinks,
  parseMarkup,
} from '~/markup-parser'
import {
  IDocument,
} from '~/arhiv/schema'
import {
  ReplicaDB,
} from '../db'
import { LockManager } from '../managers'

const log = createLogger('document')

// Active Record
export abstract class DocumentManager<T extends string, P extends object> {
  constructor(
    private _db: ReplicaDB,
    private _locks: LockManager,
    public readonly document: IDocument<T, P>,
    private _isNew: boolean,
  ) { }

  get id() {
    return this.document.id
  }

  private async _extractRefs(value: string): Promise<string[]> {
    const attachmentRefs = new Set<string>()

    const markup = parseMarkup(value)

    for (const link of selectLinks(markup)) {
      const id = link.link

      if (await this._db.getAttachment(id)) {
        attachmentRefs.add(id)
      } else {
        log.warn(`document ${this.id} references unknown entity ${id}`)
      }
    }

    return Array.from(attachmentRefs)
  }

  protected abstract _isMarkupField(field: string): boolean

  async patch(patch: Partial<P>) {
    const refSources = Object.entries(patch)
      .filter(([key]) => this._isMarkupField(key))
      .map(([, value]) => value)

    // FIXME extract doc refs
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
      deleted: true,
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
