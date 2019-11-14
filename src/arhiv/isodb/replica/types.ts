import { IDict } from '~/utils'
import {
  IDocument,
  IChangesetResult,
  IChangeset,
} from '../types'

export type LocalAttachments = IDict<Blob>

export type ChangesetExchange<T extends IDocument> = (
  changeset: IChangeset<T>,
  blobs: LocalAttachments,
) => Promise<IChangesetResult<T>>
