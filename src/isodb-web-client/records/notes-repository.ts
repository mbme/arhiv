import { INote } from '~/isodb-core/types'
import { BaseRepository } from './base-repository'
import { Note } from './note'

export class NotesRepository extends BaseRepository {
  private _wrap = (note: INote) => new Note(this._replica, this._lockAgent, note)

  createNote() {
    const id = this._replica.getRandomId()

    return this._wrap(Note.create(id))
  }

  getNotes(): Note[] {
    return this._replica.getRecords()
      .filter(Note.is)
      .map(this._wrap)
  }

  getNote(id: string): Note | undefined {
    const record = this._replica.getRecord(id)
    if (Note.is(record)) {
      return this._wrap(record)
    }

    return undefined
  }
}
