import { IDocument } from '~/isodb/types'
import { ReplicaManager } from '~/isodb/replica'

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

export type Record = INote | ITrack
export type ArhivReplica = ReplicaManager<Record>
