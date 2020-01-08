import { createLogger } from '~/logger'
import {
  nowS,
  dateNow,
  Callbacks,
} from '~/utils'
import {
  Cell,
  Observable,
  promise$,
  of$,
} from '~/reactive'
import {
  generateRandomId,
  isEmptyChangeset,
} from '../../utils'
import { LocalAttachments } from '../types'
import {
  MergeConflicts,
} from './merge-conflict'
import { TIDBStorage } from './tidb-storage'
import {
  IChangeset,
  IChangesetResponse,
  IDocument,
  IAttachment,
} from '~/arhiv/schema'

const log = createLogger('arhiv-db')

type ChangesetExchange = (changeset: IChangeset, blobs: LocalAttachments) => Promise<IChangesetResponse>

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

type SyncState =
  { type: 'initial' }
  | { type: 'sync' }
  | { type: 'merge-conflicts', conflicts: MergeConflicts }

type UpdateInfo = [number, boolean]

export class ReplicaDB {
  readonly syncState$ = new Cell<SyncState>({ type: 'initial' })
  readonly updateTime$ = new Cell<UpdateInfo>([0, false])

  private _callbacks = new Callbacks()

  constructor(
    private _storage: TIDBStorage,
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

  async getDocument(id: string): Promise<IDocument | undefined> {
    return this._storage.getDocument(id)
  }

  getDocument$(id: string): Observable<IDocument> {
    return this.updateTime$.value$
      .switchMap(() => promise$(this.getDocument(id)))
      .filter(document => !!document) as Observable<IDocument>
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

  async getDocuments(): Promise<IDocument[]> {
    return this._storage.getDocuments()
  }

  getDocuments$(): Observable<IDocument[]> {
    return this.updateTime$.value$.switchMap(() => promise$(this.getDocuments()))
  }

  async saveAttachment(file: File): Promise<string> {
    this._assertNoMergeConflicts()

    const id = await this.getRandomId()

    await this._storage.addLocalAttachment({
      id,
      rev: this.getRev(),
      createdAt: dateNow(),
      mimeType: file.type,
      size: file.size,
      deleted: false,
    }, file)
    log.info(`Created new attachment ${id} for the file "${file.name}"`)

    this._onUpdate()

    return id
  }

  async saveDocument(document: IDocument) {
    this._assertNoMergeConflicts()

    await this._storage.addLocalDocument({
      ...document,
      updatedAt: dateNow(),
    })
    log.debug(`saved document with id ${document.id}`)

    this._onUpdate(true)
  }

  isReadyToSync() {
    return this.syncState$.value.type === 'initial'
  }

  async sync(exchange: ChangesetExchange) {
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
