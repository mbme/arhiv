import {
  nowS,
} from '~/utils'
import {
  IRecord,
} from '~/isodb-core/types'
import { IsodbReplica } from '../replica'
import { LockAgent } from '../agents'
import { Attachment } from './attachment'
import { generateRandomId } from '~/isodb-core/utils'

interface ILock {
  release(): void
}

// Active Record
export abstract class BaseRecord<T extends IRecord> {
  public static create(id: string) {
    const now = nowS()

    return {
      _id: id,
      _createdTs: now,
      _updatedTs: now,
      _refs: [] as string[],
      _attachmentRefs: [] as string[],
    }
  }

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

  protected updateRefs(_value: string) {
    // FIXME implement parsing
    this._record._refs = []
    this._record._attachmentRefs = []
    this._attachments = undefined
  }

  private _getRandomAttachmentId() {
    let id: string

    do {
      id = generateRandomId()
    } while (this._replica.getAttachment(id)) // make sure generated id is free

    return id
  }

  createAttachment(file: File) {
    const id = this._getRandomAttachmentId()

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
