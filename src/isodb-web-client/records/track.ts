import {
  ITrack,
  RecordType,
} from '~/isodb-core/types'
import {
  BaseRecord,
  createRecord,
} from './base-record'

export class Track extends BaseRecord<ITrack> {
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

  public static create(id: string): ITrack {
    return {
      ...createRecord(id, RecordType.Track),
      title: '',
      artist: '',
    }
  }

  public static is(x: any): x is ITrack {
    // tslint:disable-next-line:no-unsafe-any
    return x && x._type === RecordType.Track
  }
}
