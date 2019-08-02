import { createDocument } from '~/isodb/utils'
import {
  ITrack,
  DocumentType,
} from '../types'
import {
  BaseRecord,
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
      ...createDocument(id, DocumentType.Track),
      title: '',
      artist: '',
    }
  }

  public static is(x: any): x is ITrack {
    // tslint:disable-next-line:no-unsafe-any
    return x && x._type === DocumentType.Track
  }
}
