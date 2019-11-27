import { createLogger } from '~/logger'
import {
  TIDB,
  applyForPersistentStorage,
  TIDBTransaction,
} from '~/web-tidb'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
  IChangeset,
} from '../../types'
import { LocalAttachments } from '../types'
import {
  DocumentConflict,
  MergeConflicts,
} from './merge-conflict'

const log = createLogger('arhiv-db')

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

  private async _init() {
    const [
      documents,
      attachments,
    ] = await Promise.all([
      this.getDocuments(true),
      this.getAttachments(true),
    ])

    const maxDocumentRev = documents.reduce((acc, document) => Math.max(acc, document._rev), 0)
    const maxAttachemntRev = attachments.reduce((acc, attachment) => Math.max(acc, attachment._rev), 0)

    // db rev is max document or attachment rev
    this._rev = Math.max(maxDocumentRev, maxAttachemntRev)
  }

  public static async open<T extends IDocument>() {
    const currentVersion = 1 // FIXME use version from the server

    // tslint:disable-next-line:no-shadowed-variable
    const db = await TIDB.open<IObjectStores<T>>('arhiv-replica', currentVersion, (oldVersion, db) => {
      // just to make sure we don't forget about this updater after db version increase
      if (currentVersion !== 1) {
        throw new Error('unsupported version')
      }

      if (oldVersion < 1) { // create db
        db.createObjectStore('documents', '_id')
        db.createObjectStore('documents-local', '_id')
        db.createObjectStore('attachments', '_id')
        db.createObjectStore('attachments-local', '_id')
        db.createObjectStore('attachments-data', '_id')
      }
    })
    const persistent = await applyForPersistentStorage()
    if (!persistent) {
      log.warn('Failed to apply for persistent storage')
    }

    const storage = new TIDBStorage(db)
    await storage._init()

    return storage
  }

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

  async addLocalDocument(document: T) {
    await this._idb.put('documents-local', document)
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

    const [
      localDocuments,
      documents,
    ] = await Promise.all([
      tx.store('documents-local').getAll(),
      tx.store('documents').getAll(),
    ])
    const localIds = new Set(localDocuments.map(document => document._id))

    const result = [
      ...localDocuments,
      ...documents.filter(document => !localIds.has(document._id)),
    ]

    if (withDeleted) {
      return result
    }

    return result.filter(document => !document._deleted)
  }

  async getAttachments(withDeleted = false) {
    const tx = this._idb.transaction('attachments', 'attachments-local')

    const [
      localAttachments,
      attachments,
    ] = await Promise.all([
      tx.store('attachments-local').getAll(),
      tx.store('attachments').getAll(),
    ])
    const localIds = new Set(localAttachments.map(attachment => attachment._id))

    const result = [
      ...localAttachments,
      ...attachments.filter(attachment => !localIds.has(attachment._id)),
    ]

    if (withDeleted) {
      return result
    }

    return result.filter(attachment => !attachment._deleted)
  }

  async getChangeset(): Promise<[IChangeset<T>, LocalAttachments]> {
    const tx = this._idb.transaction('documents-local', 'attachments-local', 'attachments-data')

    const [
      unusedIds,
      localDocuments,
      localAttachments,
    ] = await Promise.all([
      this._getUnusedLocalAttachmentsIds(tx),
      tx.store('documents-local').getAll(),
      tx.store('attachments-local').getAll(),
    ])

    const changeset: IChangeset<T> = {
      baseRev: this.getRev(),
      documents: localDocuments,
      attachments: localAttachments.filter(attachment => !unusedIds.includes(attachment._id)),
    }

    const attachmentsData = await Promise.all(
      changeset.attachments.map(attachment => tx.store('attachments-data').get(attachment._id)),
    )

    const localAttachmentsData: LocalAttachments = {}
    for (const data of attachmentsData) {
      if (data) {
        localAttachmentsData[data._id] = data.data
      }
    }

    return [changeset, localAttachmentsData]
  }

  async applyChangesetResult(changesetResult: IChangesetResult<T>): Promise<MergeConflicts<T> | undefined> {
    // this should never happen
    if (this._rev !== changesetResult.baseRev) {
      throw new Error(`Got rev ${changesetResult.baseRev} instead of ${this._rev}`)
    }

    // "success" means there should be no merge conflicts, so just update the data
    if (changesetResult.success) {
      await this._upgrade(changesetResult, true)

      return undefined
    }

    const conflicts = await this._getConflicts(changesetResult)
    if (!conflicts) {
      await this._upgrade(changesetResult)

      return undefined
    }

    return conflicts
  }

  private async _upgrade(changesetResult: IChangesetResult<T>, clearLocalData = false) {
    this._rev = changesetResult.currentRev

    const tx = this._idb.transactionRW(
      'documents',
      'attachments',
      'documents-local',
      'attachments-local',
      'attachments-data',
    )

    await Promise.all([
      tx.store('documents').putAll(changesetResult.documents),
      tx.store('attachments').putAll(changesetResult.attachments),
    ])

    if (clearLocalData) {
      await tx.store('documents-local').clear()

      const [
        unusedIds,
        localAttachmentIds,
      ] = await Promise.all([
        this._getUnusedLocalAttachmentsIds(tx),
        tx.store('attachments-local').getAllKeys(),
      ])

      const idsToRemove = localAttachmentIds.filter(id => !unusedIds.includes(id))
      await Promise.all([
        ...idsToRemove.map(id => tx.store('attachments-local').delete(id)),
        ...idsToRemove.map(id => tx.store('attachments-data').delete(id)),
      ])
    }
  }

  private async _getConflicts(changesetResult: IChangesetResult<T>): Promise<MergeConflicts<T> | undefined> {
    const tx = this._idb.transaction('documents', 'documents-local')

    const localDocuments = await tx.store('documents-local').getAll()

    const conflicts: Array<DocumentConflict<T>> = []
    for (const localDocument of localDocuments) {
      const remoteDocument = changesetResult.documents.find(document => document._id === localDocument._id)
      if (!remoteDocument) {
        continue
      }

      const baseDocument = await tx.store('documents').get(localDocument._id)
      if (!baseDocument) {
        throw new Error(`Can't find base document for local document ${localDocument._id}`)
      }

      conflicts.push(new DocumentConflict(baseDocument, remoteDocument, localDocument))
    }

    if (conflicts.length) {
      return new MergeConflicts(
        conflicts,
        (resolvedDocuments) => this._idb.putAll('documents-local', resolvedDocuments),
      )
    }

    return undefined
  }

  async compact(): Promise<string[]> {
    const tx = this._idb.transactionRW('documents-local', 'attachments-local', 'attachments-data')

    const unusedIds = await this._getUnusedLocalAttachmentsIds(tx)
    if (!unusedIds.length) {
      return unusedIds
    }

    await Promise.all(unusedIds.map((id) => [
      tx.store('attachments-local').delete(id),
      tx.store('attachments-data').delete(id),
    ]).flat())

    log.warn(`Removed ${unusedIds.length} unused local attachments`)

    return unusedIds
  }

  private async _getUnusedLocalAttachmentsIds(tx: TIDBTransaction<IObjectStores<T>, 'documents-local' | 'attachments-local'>) {
    const [
      documents,
      attachmentIds,
    ] = await Promise.all([
      tx.store('documents-local').getAll(),
      tx.store('attachments-local').getAllKeys(),
    ])

    const idsInUse = documents.flatMap(document => document._attachmentRefs)

    return attachmentIds.filter(id => !idsInUse.includes(id))
  }
}
