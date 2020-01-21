import { dateNow } from '~/chrono'
import { randomId } from '~/utils/random'
import {
  IChangeset,
  IDocument,
} from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export function isEmptyChangeset(changeset: IChangeset) {
  return !changeset.documents.length && !changeset.attachments.length
}

export function createDocument<T extends string, P extends object>(
  id: string,
  type: T,
  props: P,
): IDocument<T, P> {
  const now = dateNow()

  return {
    id,
    type,
    rev: 0,
    createdAt: now,
    updatedAt: now,
    refs: [] as string[],
    attachmentRefs: [] as string[],
    deleted: false,
    props,
  }
}
