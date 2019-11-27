import { createLogger } from '~/logger'
import {
  Without,
} from '~/utils'
import {
  selectLinks,
  parseMarkup,
} from '~/markup-parser'
import {
  IDocument,
  Record,
} from '../../types'
import {
  ArhivDB,
} from '../db'
import { LockManager } from '../managers'

const log = createLogger('document')

// Active Record
export class Document<T extends Record> {
  constructor(
    private _db: ArhivDB,
    private _locks: LockManager,
    public readonly record: T,
    private _isNew: boolean,
  ) { }

  get id() {
    return this.record._id
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

  async patch(patch: Partial<Without<T, keyof IDocument>>, refSource?: string) {
    const attachmentRefs = refSource === undefined
      ? this.record._attachmentRefs
      : await this._extractRefs(refSource)

    await this._db.saveDocument({
      ...this.record,
      ...patch,
      _attachmentRefs: attachmentRefs,
    })
    this._isNew = false
  }

  async delete() {
    await this._db.saveDocument({
      ...this.record,
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
