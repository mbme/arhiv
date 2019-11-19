import {
  TIDB,
} from '~/web-tidb'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
  IChangeset,
} from '../types'
import { LocalAttachments } from './types'

interface IBlob {
  _id: string
  data: Blob
}

interface IObjectStores<T extends IDocument> {
  'documents': T
  'documents-local': T
  'attachments': IAttachment
  'attachments-local': IAttachment
  'attachments-data': IBlob
}
export class TIDBStorage<T extends IDocument> {
  private _rev = 0

  private constructor(
    private _idb: TIDB<IObjectStores<T>>,
  ) { }

  getRev() {
    return this._rev
  }

  async getDocument(id: string): Promise<T | undefined> {
    const tx = this._idb.transaction('documents', 'documents-local')

    const localDocument = await tx.store('documents-local').get(id)
    if (localDocument) {
      return localDocument
    }

    return tx.store('documents').get(id)
  }

  addLocalDocument(document: T) {
    return this._idb.put('documents-local', document)
  }

  async getAttachment(id: string): Promise<IAttachment | undefined> {
    const tx = this._idb.transaction('attachments', 'attachments-local')

    const localAttachment = await tx.store('attachments-local').get(id)
    if (localAttachment) {
      return localAttachment
    }

    return tx.store('attachments').get(id)
  }

  async getLocalAttachmentData(id: string) {
    const result = await this._idb.get('attachments-data', id)

    return result?.data
  }

  async addLocalAttachment(attachment: IAttachment, file: File) {
    const tx = this._idb.transactionRW('attachments-local', 'attachments-data')

    await Promise.all([
      tx.store('attachments-local').put(attachment),
      tx.store('attachments-data').put({ _id: attachment._id, data: file }),
    ])
  }

  async getDocuments(withDeleted = false) {
    const tx = this._idb.transaction('documents', 'documents-local')

    const localDocuments = await tx.store('documents-local').getAll()
    const localIds = new Set(localDocuments.map(document => document._id))

    const documents = await tx.store('documents').getAll()

    const result = [
      ...localDocuments,
      ...documents.filter(document => !localIds.has(document._id)),
    ]

    if (withDeleted) {
      return result
    }

    return result.filter(document => !document._deleted)
  }

  async getChangeset(): Promise<[IChangeset<T>, LocalAttachments]> {
    const tx = this._idb.transaction('documents-local', 'attachments-local', 'attachments-data')

    const [
      unusedIds,
      localDocuments,
      localAttachments,
    ] = await Promise.all([
      this._getUnusedLocalAttachmentsIds(),
      tx.store('documents-local').getAll(),
      tx.store('attachments-local').getAll(),
    ]);

    const changeset: IChangeset<T> = {
      baseRev: this.getRev(),
      documents: localDocuments,
      attachments: localAttachments.filter(attachment => !unusedIds.includes(attachment._id)),
    }

    const attachmentsData = await Promise.all(changeset.attachments.map(attachment => tx.store('attachments-data').get(attachment._id)))

    const localAttachmentsData: LocalAttachments = {}
    for (const data of attachmentsData) {
      if (data) {
        localAttachmentsData[data._id] = data.data
      }
    }

    return [changeset, localAttachmentsData]
  }

  private async _getUnusedLocalAttachmentsIds() {
    const documents = await this.getDocuments(true)
    const idsInUse = documents.flatMap(document => document._attachmentRefs)

    const localAttachments = await this._idb.getAll('attachments-local')
    const localAttachmentsIds = localAttachments.map(item => item._id)

    return localAttachmentsIds.filter(id => !idsInUse.includes(id))
  }
}
