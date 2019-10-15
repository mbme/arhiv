import {
  createLogger,
  Without,
} from '~/utils'
import {
  selectLinks,
  parseMarkup,
} from '~/markup-parser'
import { IDocument } from '../isodb/types'
import {
  ArhivReplica,
  Record,
} from '../types'
import { LockManager } from '../lock-manager'

const log = createLogger('document')

// Active Record
export class Document<T extends Record> {
  constructor(
    private _replica: ArhivReplica,
    private _locks: LockManager,
    public readonly record: T,
  ) { }

  get id() {
    return this.record._id
  }

  private _extractRefs(value: string) {
    const attachmentRefs: string[] = []

    const markup = parseMarkup(value)

    for (const link of selectLinks(markup)) {
      const id = link.link

      if (this._replica.getAttachment(id)) {
        attachmentRefs.push(id)
      } else {
        log.warn(`document ${this.id} references unknown entity ${id}`)
      }
    }

    return attachmentRefs
  }

  patch(patch: Partial<Without<T, keyof IDocument>>, refSource?: string) {
    const attachmentRefs = refSource === undefined
      ? this.record._attachmentRefs
      : this._extractRefs(refSource)

    this._replica.saveDocument({
      ...this.record,
      ...patch,
      _attachmentRefs: attachmentRefs,
    })
  }

  delete() {
    this._replica.saveDocument({ // FIXME cleanup fields
      ...this.record,
      _deleted: true,
    })
  }

  isNew() {
    return !this._replica.getDocument(this.id)
  }

  isLocked$() {
    return this._locks.isDocumentLocked$(this.id)
  }

  acquireLock$() {
    return this._locks.acquireDocumentLock$(this.id)
  }
}
