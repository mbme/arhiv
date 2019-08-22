import { createDocument } from '~/isodb/utils'
import {
  INote,
  ArhivReplica,
  DocumentType,
} from '../types'
import { BaseRepository } from './base-repository'

function isNote(x: any): x is INote {
  // tslint:disable-next-line:no-unsafe-any
  return x && x._type === DocumentType.Note
}

export class NotesRepository extends BaseRepository<INote> {
  constructor(replica: ArhivReplica) {
    super(replica, isNote)
  }

  createNote() {
    const id = this._replica.getRandomId()

    return this._wrap({
      ...createDocument(id, DocumentType.Note),
      name: '',
      data: '',
    })
  }
}
