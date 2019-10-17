import { createDocument } from '../isodb/utils'
import {
  INote,
  ArhivReplica,
  DocumentType,
} from '../types'
import { DocumentsRepository } from './documents'
import { Document } from '../entities/document'
import { LockManager } from '../managers'

function isNote(x: any): x is INote {
  // tslint:disable-next-line:no-unsafe-any
  return x && x._type === DocumentType.Note
}

export type NoteDocument = Document<INote>

export class NotesRepository extends DocumentsRepository<INote> {
  constructor(replica: ArhivReplica, locks: LockManager) {
    super(replica, locks, isNote)
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
