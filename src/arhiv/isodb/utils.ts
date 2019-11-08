import { nowS } from '~/utils'
import { randomId } from '~/utils/random'
import { Observable, promise$ } from '~/reactive'
import {
  IChangeset,
  IDocument,
} from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export function isEmptyChangeset<T extends IDocument>(changeset: IChangeset<T>) {
  return !changeset.documents.length && !changeset.attachments.length
}

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

export function fetchAttachment$(id: string) {
  return new Observable<Blob>((observer) => {
    const controller = new AbortController()

    const promise = fetch(`/api/file?fileId=${id}`, {
      cache: 'force-cache',
      signal: controller.signal,
    }).then((response) => {
      if (!response.ok) {
        throw response
      }

      return response.blob()
    })

    const unsub = promise$(promise).subscribe(observer)

    return () => {
      unsub()
      controller.abort()
    }
  })
}
