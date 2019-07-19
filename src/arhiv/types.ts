import { IDocument } from '~/isodb/types'

export enum DocumentType {
  Note = 'note',
  Track = 'track',
}

export interface INote extends IDocument {
  readonly _type: DocumentType.Note
  name: string
  data: string
}

export interface ITrack extends IDocument {
  readonly _type: DocumentType.Track
  artist: string
  title: string
}
