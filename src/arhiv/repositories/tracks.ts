import { createDocument } from '../isodb/utils'
import {
  ITrack,
  DocumentType,
  ArhivReplica,
} from '../types'
import { DocumentsRepository } from './documents'
import { Document } from '../entities/document'
import { LockManager } from '../lock-manager'

function isTrack(x: any): x is ITrack {
  // tslint:disable-next-line:no-unsafe-any
  return x && x._type === DocumentType.Track
}

export type TrackDocument = Document<ITrack>

export class TracksRepository extends DocumentsRepository<ITrack> {
  constructor(replica: ArhivReplica, locks: LockManager) {
    super(replica, locks, isTrack)
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
