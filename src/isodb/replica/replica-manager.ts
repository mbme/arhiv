import {
  createLogger,
  Callbacks,
} from '~/utils'
import {
  Cell,
  Observable,
} from '~/utils/reactive'
import { IsodbReplica } from './replica'
import {
  IDocument,
  IChangeset,
  IChangesetResult,
  IAttachment,
} from '../types'
import {
  IReplicaStorage,
  LocalAttachments,
} from './replica-storage'
import { MergeConflicts } from './merge-conflict'
import {
  isEmptyChangeset,
  generateRandomId,
} from '../utils'

const log = createLogger('isodb:replica-manager')

type SyncState<T extends IDocument> = { type: 'initial' }
  | { type: 'sync' }
  | { type: 'merge-conflicts', conflicts: MergeConflicts<T> }

type ChangesetExchange<T extends IDocument> = (
  changeset: IChangeset<T>,
  blobs: LocalAttachments,
) => Promise<IChangesetResult<T>>

export class ReplicaManager<T extends IDocument> {
  readonly syncState$ = new Cell<SyncState<T>>({ type: 'initial' })

  private _replica: IsodbReplica<T>
  private _callbacks = new Callbacks()

  constructor(storage: IReplicaStorage<T>) {
    this._replica = new IsodbReplica(storage)

    this._callbacks.add(
      this.syncState$.value$.subscribe({
        next: state => log.info(`sync state -> ${state}`),
      }),

      this._replica.mergeConflicts$.value$.subscribe({
        next: (mergeConflicts) => {
          this.syncState$.value = mergeConflicts
            ? { type: 'merge-conflicts', conflicts: mergeConflicts }
            : { type: 'initial' }
        },
      }),
    )

    // run compaction on startup
    this._replica.compact()
  }

  getRandomId() {
    let id: string

    do {
      id = generateRandomId()
    } while (
      this._replica.getDocument(id)
      || this._replica.getAttachment(id)) // make sure generated id is free

    return id
  }

  getAttachment(id: string): IAttachment | undefined {
    return this._replica.getAttachment(id)
  }

  getAttachmentData$(id: string): Observable<Blob> {
    return new Observable<Blob>((observer) => {
      if (!this.getAttachment(id)) {
        throw new Error(`attachment ${id} doesn't exist`)
      }

      const data = this._replica.getLocalAttachmentData(id)

      if (data) {
        observer.next(data)
        observer.complete()

        return undefined
      }

      const controller = new AbortController()

      fetch(`/api/file?fileId=${id}`, {
        cache: 'force-cache',
        signal: controller.signal,
      }).then((response) => {
        if (!response.ok) {
          throw response
        }

        return response.blob()
      }).then(
        observer.next,
      ).catch((e) => {
        log.error(`Failed to fetch attachment ${id}`, e)
        observer.error(e)
      })

      return () => {
        controller.abort()
      }
    })
  }

  saveAttachment(file: File): string {
    const id = this.getRandomId()
    this._replica.saveAttachment(id, file)
    log.info(`Created new attachment ${id} for the file "${file.name}"`)

    return id
  }

  getDocuments$(): Observable<T[]> {
    return this._replica.updateTime$.value$.map(() => this._replica.getDocuments())
  }

  getDocument$(id: string): Observable<T | undefined> {
    return this._replica.updateTime$.value$.map(() => this._replica.getDocument(id))
  }

  getDocument(id: string): T | undefined {
    return this._replica.getDocument(id)
  }

  saveDocument(document: T) {
    this._replica.saveDocument(document)
  }

  async sync(exchange: ChangesetExchange<T>) {
    if (!this.isReadyToSync()) {
      throw new Error('not ready to sync')
    }

    try {
      log.debug('sync: starting')

      this.syncState$.value = { type: 'sync' }

      const [changeset, localAttachments] = this._replica.getChangeset()

      if (isEmptyChangeset(changeset)) {
        log.info('sync: sending empty changeset')
      } else {
        // tslint:disable-next-line:max-line-length
        log.info(`sync: sending ${changeset.documents.length} documents, ${changeset.attachments.length} attachments, (${Object.keys(localAttachments).length} BLOBs)`)
      }

      const result = await exchange(changeset, localAttachments)

      // tslint:disable-next-line:max-line-length
      log.info(`sync: success: ${result.success}, got ${result.documents.length} documents and ${result.attachments.length} attachments`)

      await this._replica.applyChangesetResult(result)

      if (this._replica.hasMergeConflicts()) {
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

  isReadyToSync() {
    return this.syncState$.value.type === 'initial'
  }

  stop() {
    this._callbacks.runAll()
    // TODO stop storage?
  }
}
