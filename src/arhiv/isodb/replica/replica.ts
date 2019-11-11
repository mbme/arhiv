import {
  createLogger,
  nowS,
  Callbacks,
  Procedure,
} from '~/utils'
import {
  Cell,
  Observable,
  promise$,
  of$,
} from '~/reactive'
import {
  IAttachment,
  IDocument,
  IChangesetResult,
  IChangeset,
} from '../types'
import {
  IReplicaStorage,
  ChangesetExchange,
  LocalAttachments,
} from './types'
import { MergeConflicts, DocumentConflict } from './merge-conflict'
import {
  generateRandomId,
  isEmptyChangeset,
  fetchAttachment$,
} from '../utils'

const log = createLogger('isodb-replica')

type SyncState<T extends IDocument> =
  { type: 'initial' }
  | { type: 'sync' }
  | { type: 'merge-conflicts', conflicts: MergeConflicts<T> }

export class IsodbReplica<T extends IDocument> {
  readonly syncState$ = new Cell<SyncState<T>>({ type: 'initial' })
  readonly updateTime$ = new Cell(0)

  private _callbacks = new Callbacks()

  constructor(
    private _storage: IReplicaStorage<T>,
    private _onLocalUpdate: Procedure,
  ) {
    this._callbacks.add(
      this.syncState$.value$.subscribe({
        next: state => log.debug(`sync state -> ${state.type}`),
      }),
    )
  }

  getRev() {
    return this._storage.getRev()
  }

  async getRandomId() {
    let id: string

    do {
      id = generateRandomId()
    } while (await this.getDocument(id) || await this.getAttachment(id)) // make sure generated id is free

    return id
  }

  async getDocument(id: string): Promise<T | undefined> {
    const localDocument = await this._storage.getLocalDocument(id)

    if (localDocument) {
      return localDocument
    }

    return this._storage.getDocument(id)
  }

  getDocument$(id: string): Observable<T> {
    return this.updateTime$.value$
      .switchMap(() => promise$(this.getDocument(id)))
      .filter(document => !!document) as Observable<T>
  }

  async getAttachment(id: string): Promise<IAttachment | undefined> {
    const localAttachment = await this._storage.getLocalAttachment(id)

    if (localAttachment) {
      return localAttachment
    }

    return this._storage.getAttachment(id)
  }

  getAttachmentData$(id: string): Observable<Blob> {
    return promise$(
      this.getAttachment(id).then((attachment) => {
        if (!attachment) {
          throw new Error(`attachment ${id} doesn't exist`)
        }

        return this._storage.getLocalAttachmentData(id)
      }),
    ).switchMap((data) => {
      if (data) {
        return of$(data)
      }

      return fetchAttachment$(id)
    })
  }

  async getDocuments(includeDeleted = false): Promise<T[]> {
    const localDocuments = await this._storage.getLocalDocuments()
    const localIds = new Set(localDocuments.map(item => item._id))

    const documents = (await this._storage.getDocuments()).filter(item => !localIds.has(item._id))

    const result = [
      ...documents,
      ...localDocuments,
    ]

    if (includeDeleted) {
      return result
    }

    return result.filter(document => !document._deleted)
  }

  getDocuments$(): Observable<T[]> {
    return this.updateTime$.value$.switchMap(() => promise$(this.getDocuments()))
  }

  async saveAttachment(file: File): Promise<string> {
    this._assertNoMergeConflicts()

    const id = await this.getRandomId()

    await this._storage.addLocalAttachment({
      _id: id,
      _rev: this.getRev(),
      _createdTs: nowS(),
      _mimeType: file.type,
      _size: file.size,
    }, file)
    log.info(`Created new attachment ${id} for the file "${file.name}"`)

    this._onUpdate()

    return id
  }

  async saveDocument(document: T) {
    this._assertNoMergeConflicts()

    await this._storage.addLocalDocument({
      ...document,
      _updatedTs: nowS(),
    })
    log.debug(`saved document with id ${document._id}`)

    this._onUpdate()
    this._onLocalUpdate()
  }

  isReadyToSync() {
    return this.syncState$.value.type === 'initial'
  }

  async sync(exchange: ChangesetExchange<T>) {
    if (!this.isReadyToSync()) {
      throw new Error('not ready to sync')
    }

    try {
      log.debug('sync: starting')

      this.syncState$.value = { type: 'sync' }

      const [changeset, localAttachments] = await this._getChangeset()

      if (isEmptyChangeset(changeset)) {
        log.info('sync: sending empty changeset')
      } else {
        // tslint:disable-next-line:max-line-length
        log.info(`sync: sending ${changeset.documents.length} documents, ${changeset.attachments.length} attachments, (${Object.keys(localAttachments).length} BLOBs)`)
      }

      const result = await exchange(changeset, localAttachments)

      // tslint:disable-next-line:max-line-length
      log.info(`sync: success: ${result.success}, got ${result.documents.length} documents and ${result.attachments.length} attachments`)

      await this._applyChangesetResult(result)

      if (this._hasMergeConflicts()) {
        log.debug('sync: merge conflicts')

        return false
      }

      log.debug('sync: ok')

      this.syncState$.value = { type: 'initial' }

      return true

    } catch (e) {
      log.error('Failed to sync', e)

      this.syncState$.value = { type: 'initial' }

      return false
    }
  }

  private _hasMergeConflicts() {
    return this.syncState$.value.type === 'merge-conflicts'
  }

  private _assertNoMergeConflicts() {
    if (this._hasMergeConflicts()) {
      throw new Error('there is a pending merge conflict')
    }
  }

  private _onUpdate() {
    this.updateTime$.value = nowS()
  }

  private async _applyChangesetResult(changesetResult: IChangesetResult<T>) {
    // this should never happen
    if (this.getRev() !== changesetResult.baseRev) {
      throw new Error(`Got rev ${changesetResult.baseRev} instead of ${this.getRev()}`)
    }

    // "success" means there should be no merge conflicts, so just update the data
    if (changesetResult.success) {
      await this._storage.upgrade(changesetResult)
      await this._clearLocalData()
      this._onUpdate()

      return
    }

    const conflicts: Array<DocumentConflict<T>> = []
    for (const localDocument of await this._storage.getLocalDocuments()) {
      const remoteDocument = changesetResult.documents.find(document => document._id === localDocument._id)
      if (!remoteDocument) {
        continue
      }

      const baseDocument = await this._storage.getDocument(localDocument._id)
      if (!baseDocument) {
        throw new Error(`Can't find base document for local document ${localDocument._id}`)
      }

      conflicts.push(new DocumentConflict(baseDocument, remoteDocument, localDocument))
    }

    if (conflicts.length) {
      const mergeConflicts = new MergeConflicts(conflicts)
      this.syncState$.value = { type: 'merge-conflicts', conflicts: mergeConflicts }

      const resolvedDocuments = await mergeConflicts.promise
      // save resolved versions of the documents
      for (const document of resolvedDocuments) {
        await this._storage.addLocalDocument(document)
      }

      await this._storage.upgrade(changesetResult)
      this._onUpdate()

      this.syncState$.value = { type: 'initial' }
    } else {
      await this._storage.upgrade(changesetResult)
      this._onUpdate()
    }
  }

  private async _getUnusedLocalAttachmentsIds() {
    const idsInUse = (await this.getDocuments()).flatMap(document => document._attachmentRefs)
    const localAttachmentsIds = (await this._storage.getLocalAttachments()).map(item => item._id)

    return localAttachmentsIds.filter(id => !idsInUse.includes(id))
  }

  private async _getChangeset(): Promise<[IChangeset<T>, LocalAttachments]> {
    const unusedIds = await this._getUnusedLocalAttachmentsIds()

    const changeset: IChangeset<T> = {
      baseRev: this._storage.getRev(),
      documents: await this._storage.getLocalDocuments(),
      attachments: (await this._storage.getLocalAttachments())
        .filter(attachment => !unusedIds.includes(attachment._id)),
    }

    const localAttachmentsData: LocalAttachments = {}
    for (const attachment of changeset.attachments) {
      const data = await this._storage.getLocalAttachmentData(attachment._id)
      if (data) {
        localAttachmentsData[attachment._id] = data
      }
    }

    return [changeset, localAttachmentsData]
  }

  private async _clearLocalData() {
    for (const localDocument of await this._storage.getLocalDocuments()) {
      await this._storage.removeLocalDocument(localDocument._id)
    }

    const unusedIds = await this._getUnusedLocalAttachmentsIds()
    for (const localAttachment of await this._storage.getLocalAttachments()) {
      if (!unusedIds.includes(localAttachment._id)) {
        await this._storage.removeLocalAttachment(localAttachment._id)
      }
    }
  }

  /**
   * Remove unused local attachments
   */
  async compact() {
    const unusedIds = await this._getUnusedLocalAttachmentsIds()
    if (!unusedIds.length) {
      return
    }

    for (const id of unusedIds) {
      await this._storage.removeLocalAttachment(id)
      log.warn(`Removing unused local attachment ${id}`)
    }

    this._onUpdate()
  }

  stop() {
    this._callbacks.runAll(true)
    // TODO stop storage?
  }
}
