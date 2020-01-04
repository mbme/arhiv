import { dateNow } from '~/utils'
import { randomId } from '~/utils/random'
import {
  IChangeset,
  MarkupString,
  INote,
  ITrack,
  ArhivDocumentType,
  ArhivDocument,
} from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export function isEmptyChangeset(changeset: IChangeset) {
  return !changeset.documents.length && !changeset.attachments.length
}

export function createDocument<T extends ArhivDocumentType>(id: string, type: T)
  : T extends 'note' ? INote
  : T extends 'track' ? ITrack
  : never

export function createDocument(id: string, type: ArhivDocumentType): ArhivDocument {
  const now = dateNow()

  if (type === 'note') {
    return {
      _id: id,
      _rev: 0,
      _createdAt: now,
      _updatedAt: now,
      _attachmentRefs: [] as string[],
      _type: 'note',
      name: '',
      data: new MarkupString(''),
    }
  }

  if (type === 'track') {
    return {
      _id: id,
      _rev: 0,
      _createdAt: now,
      _updatedAt: now,
      _attachmentRefs: [] as string[],
      _type: 'track',
      title: '',
      artist: '',
    }
  }

  throw new Error(`unexpected document type: ${type}`)
}
