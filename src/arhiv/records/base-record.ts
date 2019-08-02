import { createLogger } from '~/logger'
import {
  markupParser,
  selectLinks,
} from '~/markup-parser'
import { stringifyFailure } from '~/parser-combinator'
import { LockAgent } from '../agents'
import { Attachment } from './attachment'
import {
  ArhivReplica,
  Record,
} from '../types';

const log = createLogger('record')

interface ILock {
  release(): void
}

// Active Record
export abstract class BaseRecord<T extends Record> {
  protected _record: T
  private _attachments?: Attachment[]
  private _hasNewAttachments = false
  public lock?: ILock

  constructor(
    protected _replica: ArhivReplica,
    protected _lockAgent: LockAgent,
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
    const id = this._replica.saveAttachment(file)
    this._hasNewAttachments = true

    return id
  }

  save() {
    this._replica.saveDocument(this._record)
    this._hasNewAttachments = false
  }

  isNew() {
    return !this._replica.getDocument(this.id)
  }

  isLocked() {
    return this._lockAgent.isRecordLocked(this.id)
  }

  acquireLock() {
    this._lockAgent.lockRecord(this.id)
    this.lock = {
      release: () => {
        this._lockAgent.unlockRecord(this.id)
        this.lock = undefined

        // remove unused new attachments if record wasn't saved
        if (this._hasNewAttachments) {
          this._replica.compact()
        }
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
