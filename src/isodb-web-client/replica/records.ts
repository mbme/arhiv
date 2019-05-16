
export class Note extends BaseRecord<INote> {
  public static create(id: string): INote {
    return {
      ...BaseRecord.create(id),
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

  static is(x: any): x is Note {
    return x instanceof Note
  }
}

export class Track extends BaseRecord<ITrack> {
  public static create(id: string): ITrack {
    return {
      ...BaseRecord.create(id),
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

  static is(x: any): x is Track {
    return x instanceof Track
  }
}

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
