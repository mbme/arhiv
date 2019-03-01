import {
  nowS,
} from '~/utils'
import {
  IRecord,
  INote,
  ITrack,
  RecordType,
  IAttachment,
} from '~/isodb-core/types'
import { generateRecordId } from '~/isodb-core/utils'
import IsodbReplica from './replica'

// Active Record
abstract class BaseRecord<T extends IRecord> {
  protected _record: T
  private _attachments: Attachment[] | undefined

  constructor(
    protected _replica: IsodbReplica,
    record?: T,
  ) {
    this._record = record || this._create()
  }

  protected updateRefs(_value: string) {
    // FIXME implement parsing
    this._record._refs = []
    this._record._attachmentRefs = []
    this._attachments = undefined
  }

  protected abstract _create(): T

  private _getRandomId() {
    let id: string

    do {
      id = generateRecordId()
    } while (this._replica.getRecord(id)) // make sure generated id is free

    return id
  }

  protected _createRecord() {
    const now = nowS()

    return {
      _id: this._getRandomId(),
      _createdTs: now,
      _updatedTs: now,
      _refs: [],
      _attachmentRefs: [],
    }
  }

  save() {
    this._replica.saveRecord({
      ...this._record,
      _updatedTs: nowS(),
    })
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

  get refs(): ReadonlyArray<string> {
    return this._record._refs
  }

  get attachments(): Attachment[] {
    this._attachments = this._attachments || this._record._attachmentRefs.map(id => {
      const attachment = this._replica.getAttachment(id)
      if (!attachment) {
        throw new Error(`record ${this._record._id} references unknown attachment ${id}`)
      }

      return attachment
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

export class Note extends BaseRecord<INote> {
  _create(): INote {
    return {
      ...this._createRecord(),
      _type: RecordType.Note,
      name: '',
      data: '',
    }
  }

  get name() {
    return this._record.name
  }

  set name(value: string) {
    this._record.name = value
  }

  get data() {
    return this._record.data
  }

  set data(value: string) {
    this._record.data = value
    this.updateRefs(value)
  }
}

export class Track extends BaseRecord<ITrack> {
  _create(): ITrack {
    return {
      ...this._createRecord(),
      _type: RecordType.Track,
      title: '',
      artist: '',
    }
  }

  get title() {
    return this._record.title
  }

  set title(value: string) {
    this._record.title = value
  }

  get artist() {
    return this._record.artist
  }

  set artist(value: string) {
    this._record.artist = value
  }
}

export type Record = Note | Track

export class Attachment {
  constructor(
    private _replica: IsodbReplica,
    private _attachment: IAttachment,
  ) { }

  get url() {
    return this._replica.getAttachmentUrl(this._attachment._id)
  }

  get id() {
    return this._attachment._id
  }

  save() {
    this._replica.saveAttachment(this._attachment)
  }
}
