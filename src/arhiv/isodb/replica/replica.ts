import {
  createLogger,
  nowS,
  Callbacks,
} from '~/utils'
import {
  Cell,
  Observable,
} from '~/utils/reactive'
import {
  IAttachment,
  IDocument,
  IChangesetResult,
} from '../types'
import {
  IReplicaStorage,
  ChangesetExchange,
} from './types'
import { MergeConflicts } from './merge-conflict'
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
  ) {
    this._callbacks.add(
      this.syncState$.value$.subscribe({
        next: state => log.info(`sync state -> ${state.type}`),
      }),
    )

    // run compaction on startup
    this._compact()
  }

  getRev() {
    return this._storage.getRev()
  }

  getRandomId() {
    let id: string

    do {
      id = generateRandomId()
    } while (
      this.getDocument(id)
      || this.getAttachment(id)) // make sure generated id is free

    return id
  }

  getDocument(id: string): T | undefined {
    return this._storage.getLocalDocument(id) || this._storage.getDocument(id)
  }

  getDocument$(id: string): Observable<T | undefined> {
    return this.updateTime$.value$.map(() => this.getDocument(id))
  }

  getAttachment(id: string): IAttachment | undefined {
    return this._storage.getLocalAttachment(id) || this._storage.getAttachment(id)
  }

  getAttachmentData$(id: string): Observable<Blob> {
    return new Observable<Blob>((observer) => {
      if (!this.getAttachment(id)) {
        throw new Error(`attachment ${id} doesn't exist`)
      }

      const data = this._storage.getLocalAttachmentData(id)

      if (data) {
        observer.next(data)
        observer.complete()

        return undefined
      }

      return fetchAttachment$(id).subscribe(observer)
    })
  }

  getDocuments(includeDeleted = false): T[] {
    const localDocuments = this._storage.getLocalDocuments()
    const localIds = new Set(localDocuments.map(item => item._id))

    const documents = this._storage.getDocuments().filter(item => !localIds.has(item._id))

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
    return this.updateTime$.value$.map(() => this.getDocuments())
  }

  saveAttachment(file: File): string {
    this._assertNoMergeConflicts()

    const id = this.getRandomId()

    this._storage.addLocalAttachment({
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

  saveDocument(document: T) {
    this._assertNoMergeConflicts()

    this._storage.addLocalDocument({
      ...document,
      _updatedTs: nowS(),
    })
    log.debug(`saved document with id ${document._id}`)

    this._onUpdate()
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

      const [changeset, localAttachments] = this._storage.getChangeset()

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
      this._storage.clearLocalData()

      this._storage.upgrade(changesetResult)
      this._onUpdate()

      return
    }

    const mergeConflicts = new MergeConflicts<T>((documents) => {
      // save resolved versions of the documents
      for (const document of documents) {
        this._storage.addLocalDocument(document)
      }

      this._storage.upgrade(changesetResult)
      this._onUpdate()

      this.syncState$.value = { type: 'initial' }
    })

    for (const localDocument of this._storage.getLocalDocuments()) {
      const remoteDocument = changesetResult.documents.find(document => document._id === localDocument._id)
      if (!remoteDocument) {
        continue
      }

      const baseDocument = this._storage.getDocument(localDocument._id)
      if (!baseDocument) {
        throw new Error(`Can't find base document for local document ${localDocument._id}`)
      }

      mergeConflicts.addConflict(baseDocument, remoteDocument, localDocument)
    }

    if (mergeConflicts.conflicts.length) {
      this.syncState$.value = { type: 'merge-conflicts', conflicts: mergeConflicts }
    } else {
      this._storage.upgrade(changesetResult)
      this._onUpdate()
    }
  }

  /**
   * Remove unused local attachments
   */
  private _compact() {
    const idsInUse = new Set(this._storage.getDocuments().flatMap(document => document._attachmentRefs))
    const localAttachmentIds = new Set(this._storage.getLocalAttachments().map(item => item._id))

    let updated = false

    for (const id of localAttachmentIds) {
      // remove unused new local attachments
      if (!idsInUse.has(id)) {
        log.warn(`Removing unused local attachment ${id}`)
        this._storage.removeLocalAttachment(id)
        updated = true
      }
    }

    if (updated) {
      this._onUpdate()
    }
  }

  stop() {
    this._callbacks.runAll()
    // TODO stop storage?
  }
}
