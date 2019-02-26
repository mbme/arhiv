import {
  nowS,
} from '~/utils'
import { IRecord } from './types'

enum RecordType {
  Note = 'note',
  Track = 'track',
}


// Record types
export interface INote extends IRecord {
  readonly _type: RecordType.Note
  name: string
  data: string
}

export interface ITrack extends IRecord {
  readonly _type: RecordType.Track
  artist: string
  title: string
}

function createRecord(refs = [], attachmentRefs = []) {
  const now = nowS()

  return {
    _id: getRandomId(),
    _createdTs: now,
    _updatedTs: now,
    _refs: refs,
    _attachmentRefs: attachmentRefs,
  }
}

function updateRecord(refs = [], attachmentRefs = []) {
  return {
    _updatedTs: nowS(),
    _refs: refs,
    _attachmentRefs: attachmentRefs,
  }
}

export function createNote(name: string, data: string): INote {
  return {
    ...createRecord(),
    _type: 'note',
    name,
    data,
  }
}

export function updateNote(note: INote, name: string, data: string): INote {
  return {
    ...note,
    ...updateRecord(),
    name,
    data,
  }
}

export function createTrack(artist: string, title: string): ITrack {
  return {
    ...createRecord(),
    _type: 'track',
    artist,
    title,
  }
}

export function updateTrack(track: ITrack, artist: string, title: string): ITrack {
  return {
    ...track,
    ...updateRecord(),
    title,
    artist,
  }
}
