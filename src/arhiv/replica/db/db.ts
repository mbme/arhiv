import { createLogger } from '~/logger'
import {
  nowS,
  Callbacks,
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
  IChangesetResponse,
  IChangeset,
  Record,
} from '../../types'
import {
  generateRandomId,
  isEmptyChangeset,
} from '../../utils'
import { LocalAttachments } from '../types'
import {
  MergeConflicts,
} from './merge-conflict'
import { TIDBStorage } from './tidb-storage'

const log = createLogger('arhiv-db')

type ChangesetExchange<T extends IDocument> = (
  changeset: IChangeset<T>,
  blobs: LocalAttachments,
) => Promise<IChangesetResponse<T>>

function fetchAttachment$(id: string) {
  return new Observable<Blob>((observer) => {
    const controller = new AbortController()

    const promise = fetch(`/api/file/${id}`, {
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

type SyncState<T extends IDocument> =
  { type: 'initial' }
  | { type: 'sync' }
  | { type: 'merge-conflicts', conflicts: MergeConflicts<T> }

type UpdateInfo = [number, boolean]

export class ArhivDB<T extends IDocument = Record> {
  readonly syncState$ = new Cell<SyncState<T>>({ type: 'initial' })
  readonly updateTime$ = new Cell<UpdateInfo>([0, false])

  private _callbacks = new Callbacks()

  constructor(
    private _storage: TIDBStorage<T>,
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
    return this._storage.getDocument(id)
  }

  getDocument$(id: string): Observable<T> {
    return this.updateTime$.value$
      .switchMap(() => promise$(this.getDocument(id)))
      .filter(document => !!document) as Observable<T>
  }

  async getAttachment(id: string): Promise<IAttachment | undefined> {
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

  async getDocuments(): Promise<T[]> {
    return this._storage.getDocuments()
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

    this._onUpdate(true)
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

      const [changeset, localAttachments] = await this._storage.getChangeset()

      if (isEmptyChangeset(changeset)) {
        log.info('sync: sending empty changeset')
      } else {
        // tslint:disable-next-line:max-line-length
        log.info(`sync: sending ${changeset.documents.length} documents, ${changeset.attachments.length} attachments, (${Object.keys(localAttachments).length} BLOBs)`)
      }

      const result = await exchange(changeset, localAttachments)

      // tslint:disable-next-line:max-line-length
      log.info(`sync: ${result.status}, got ${result.documents.length} documents and ${result.attachments.length} attachments`)

      const conflicts = await this._storage.applyChangesetResponse(result)
      if (conflicts) {
        log.debug('sync: merge conflicts')

        this.syncState$.value = { type: 'merge-conflicts', conflicts }

        await conflicts.promise
      } else {
        log.debug('sync: ok')
      }

      this._onUpdate()
      this.syncState$.value = { type: 'initial' }

      return true

    } catch (e) {
      log.warn('Failed to sync', e)

      this.syncState$.value = { type: 'initial' }

      return false
    }
  }

  private _assertNoMergeConflicts() {
    if (this.syncState$.value.type === 'merge-conflicts') {
      throw new Error('there is a pending merge conflict')
    }
  }

  private _onUpdate(isLocal: boolean = false) {
    this.updateTime$.value = [nowS(), isLocal]
  }

  /**
   * Remove unused local attachments
   */
  async compact() {
    const unusedIds = await this._storage.compact()

    if (unusedIds.length) {
      this._onUpdate()
    }
  }

  stop() {
    this._callbacks.runAll(true)
    // TODO stop storage?
  }
}
