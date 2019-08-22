import { createDocument } from '~/isodb/utils'
import {
  ITrack,
  DocumentType,
  ArhivReplica,
} from '../types'
import { BaseRepository } from './base-repository'

function isTrack(x: any): x is ITrack {
  // tslint:disable-next-line:no-unsafe-any
  return x && x._type === DocumentType.Track
}

export class TracksRepository extends BaseRepository<ITrack> {
  constructor(replica: ArhivReplica) {
    super(replica, isTrack)
  }

  createTrack() {
    const id = this._replica.getRandomId()

    return this._wrap({
      ...createDocument(id, DocumentType.Track),
      title: '',
      artist: '',
    })
  }
}
