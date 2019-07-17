import { randomId } from '~/randomizer'
import { nowS } from '~/utils'
import {
  IChangeset,
  DocumentType,
  IDocument,
} from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export const isEmptyChangeset = (changeset: IChangeset) => !changeset.documents.length && !changeset.attachments.length

export function createDocument<T extends DocumentType>(id: string, type: T): IDocument {
  const now = nowS()

  return {
    _id: id,
    _rev: 1, // FIXME
    _type: type,
    _createdTs: now,
    _updatedTs: now,
    _refs: [] as string[],
    _attachmentRefs: [] as string[],
  }
}
