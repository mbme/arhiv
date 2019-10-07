import { IDocument } from './isodb/types'
import { IsodbReplica } from './isodb/replica'

export enum DocumentType {
  Note = 'note',
  Track = 'track',
}

export interface INote extends IDocument {
  readonly _type: DocumentType.Note
  readonly name: string
  readonly data: string
}

export interface ITrack extends IDocument {
  readonly _type: DocumentType.Track
  readonly artist: string
  readonly title: string
}

export type Record = INote | ITrack
export type ArhivReplica = IsodbReplica<Record>
