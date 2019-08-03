import { createLogger } from '~/logger'
import {
  markupParser,
  selectLinks,
} from '~/markup-parser'
import { stringifyFailure } from '~/parser-combinator'
import { Attachment } from './attachment'
import {
  ArhivReplica,
  Record,
} from '../types'

const log = createLogger('record')

interface ILock {
  release(): void
}

// Active Record
export abstract class BaseRecord<T extends Record> {
  protected _record: T
  private _attachments?: Attachment[]
  public lock?: ILock

  constructor(
    protected _replica: ArhivReplica,
    record: T,
  ) {
    this._record = { ...record }
  }

  protected _updateRefs(value: string) {
    const attachmentRefs: string[] = []

    const result = markupParser.parseAll(value)
    if (!result.success) {
      throw new Error(`Failed to parse markup: ${stringifyFailure(result)}`)
    }

    for (const link of selectLinks(result.result)) {
      const id = link.value[0]

      if (this._replica.getAttachment(id)) {
        attachmentRefs.push(id)
        continue
      }

      log.warn(`record ${this.id} references unknown entity ${id}`)
    }

    this._record._attachmentRefs = attachmentRefs
    this._attachments = undefined
  }

  createAttachment(file: File): string {
    return this._replica.saveAttachment(file)
  }

  save() {
    this._replica.saveDocument(this._record)
  }

  isNew() {
    return !this._replica.getDocument(this.id)
  }

  isLocked() {
    return this._replica.locks.isDocumentLocked(this.id)
  }

  acquireLock() {
    this._replica.locks.lockDocument(this.id)

    this.lock = {
      release: () => {
        this._replica.locks.unlockDocument(this.id)
        this.lock = undefined
      },
    }
  }

  get id() {
    return this._record._id
  }

  get type() {
    return this._record._type
  }

  get rev() {
    return this._record._rev
  }

  get attachments(): Attachment[] {
    this._attachments = this._attachments || this._record._attachmentRefs.map(id => {
      const attachment = this._replica.getAttachment(id)
      if (!attachment) {
        throw new Error(`record ${this._record._id} references unknown attachment ${id}`)
      }

      return new Attachment(this._replica, attachment)
    })

    return this._attachments
  }

  get deleted(): boolean {
    return this._record._deleted || false
  }

  set deleted(value: boolean) {
    this._record._deleted = value
  }

  get createdTs() {
    return this._record._createdTs
  }

  get updatedTs() {
    return this._record._updatedTs
  }
}
