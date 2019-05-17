import {
  nowS,
} from '~/utils'
import {
  IRecord,
} from '~/isodb-core/types'
import { IsodbReplica } from '../replica'
import { Attachment } from './attachment'

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

  private _attachments?: Attachment[]

  constructor(
    protected _replica: IsodbReplica,
    protected _record: T,
  ) { }

  protected updateRefs(_value: string) {
    // FIXME implement parsing
    this._record._refs = []
    this._record._attachmentRefs = []
    this._attachments = undefined
  }

  save() {
    this._replica.saveRecord({
      ...this._record,
      _updatedTs: nowS(),
    })
    // FIXME save attachments?
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
