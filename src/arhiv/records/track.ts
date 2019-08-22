import { createDocument } from '~/isodb/utils'
import {
  ITrack,
  DocumentType,
} from '../types'
import {
  BaseRecord,
} from './base-record'

export class Track extends BaseRecord<ITrack> {
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
