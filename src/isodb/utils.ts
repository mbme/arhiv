import { randomId } from '~/randomizer'
import { nowS } from '~/utils'
import { IChangeset } from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export const isEmptyChangeset = (changeset: IChangeset) => !changeset.documents.length && !changeset.attachments.length

export function createDocument<T extends string>(id: string, type: T) {
  const now = nowS()

  return {
    _id: id,
    _rev: 0,
    _type: type,
    _createdTs: now,
    _updatedTs: now,
    _attachmentRefs: [] as string[],
  }
}
