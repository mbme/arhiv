import { nowS } from '~/utils'
import { createLogger } from '~/logger'
import {
  IRecord,
  RecordType,
} from '~/isodb-core/types'
import {
  markupParser,
  selectLinks,
} from '~/markup-parser'
import { stringifyFailure } from '~/parser-combinator'
import { IsodbReplica } from '../replica'
import { LockAgent } from '../agents'
import { Attachment } from './attachment'

const log = createLogger('record')

interface ILock {
  release(): void
}

export function createRecord<T extends RecordType>(id: string, type: T) {
  const now = nowS()

  return {
    _id: id,
    _type: type,
    _createdTs: now,
    _updatedTs: now,
    _refs: [] as string[],
    _attachmentRefs: [] as string[],
  }
}

// Active Record
export abstract class BaseRecord<T extends IRecord> {
  protected _record: T
  private _attachments?: Attachment[]
  private _hasNewAttachments = false
  public lock?: ILock

  constructor(
    protected _replica: IsodbReplica,
    protected _lockAgent: LockAgent,
    record: T,
  ) {
    this._record = { ...record }
  }

  protected _updateRefs(value: string) {
    const refs: string[] = []
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

      if (this._replica.getRecord(id)) {
        refs.push(id)
        continue
      }

      log.warn(`record ${this.id} references unknown entity ${id}`)
    }

    this._record._refs = refs
    this._record._attachmentRefs = attachmentRefs
    this._attachments = undefined
  }

  createAttachment(file: File) {
    const id = this._replica.getRandomId()

    this._replica.saveAttachment({ _id: id }, file)
    this._hasNewAttachments = true

    return id
  }

  save() {
    this._replica.saveRecord({
      ...this._record,
      _updatedTs: nowS(),
    })
    this._hasNewAttachments = false
  }

  isNew() {
    return !this._replica.getRecord(this.id)
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

  get refs(): readonly string[] {
    return this._record._refs
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

  get deleted() {
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
