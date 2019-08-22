import { createLogger } from '~/logger'
import { ReactiveValue } from '~/utils/reactive'
import { LockManager } from './lock-manager'
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
} from './storage'
import {
  isEmptyChangeset,
  generateRandomId,
} from '../utils'

const log = createLogger('isodb:replica-manager')

type SyncState = 'sync' | 'merge-conflicts' | 'merge-conflicts-resolved' | 'synced' | 'not-synced'

type ChangesetExchange<T extends IDocument> = (
  changeset: IChangeset<T>,
  blobs: LocalAttachments,
) => Promise<IChangesetResult<T>>

export class ReplicaManager<T extends IDocument> {
  locks = new LockManager()
  private _replica: IsodbReplica<T>

  $syncState = new ReactiveValue<SyncState>('not-synced')

  constructor(storage: IReplicaStorage<T>) {
    this._replica = new IsodbReplica(storage)

    this._replica.$mergeConflicts.subscribe((mergeConflicts) => {
      if (mergeConflicts) {
        this.$syncState.next('merge-conflicts')
      } else if (this.$syncState.currentValue === 'merge-conflicts') {
        this.$syncState.next('merge-conflicts-resolved')
      }
    })

    // run compaction if db isn't locked
    this.locks.$state.subscribe((lockState) => {
      if (lockState.type === 'free') {
        this._replica.compact()
      }
    })
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

  getAttachment(id: string): ReactiveValue<IAttachment | undefined> {
    return this._replica.$updateTime.map(() => this._replica.getAttachment(id))
  }

  getAttachmentUrl(id: string): ReactiveValue<string | undefined> {
    return this._replica.$updateTime.map(() => this._replica.getAttachmentUrl(id))
  }

  saveAttachment(file: File) {
    const id = this.getRandomId()
    this._replica.saveAttachment(id, file)

    return id
  }

  getDocuments(): ReactiveValue<T[]> {
    return this._replica.$updateTime.map(() => this._replica.getDocuments())
  }

  getDocument(id: string): ReactiveValue<T | undefined> {
    return this._replica.$updateTime.map(() => this._replica.getDocument(id))
  }

  saveDocument(document: T) {
    this._replica.saveDocument(document)
  }

  async sync(exchange: ChangesetExchange<T>): Promise<boolean> {
    if (!this.locks.isFree()) {
      log.debug('Skipping sync: lock is not free')

      return false
    }

    if (this._replica.hasMergeConflicts()) {
      log.debug('Skipping sync: pending merge conflicts')

      return false
    }

    try {
      log.debug('sync: starting')

      this.$syncState.next('sync')

      this.locks.lockDB()

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

      this.$syncState.next('synced')

      return true

    } catch (e) {
      log.error('Failed to sync', e)

      this.$syncState.next('not-synced')

      return false
    } finally {
      this.locks.unlockDB()
    }
  }

  stop() {
    this.locks.stop()
    this.$syncState.complete()
    this._replica.stop()
    // TODO stop storage?
  }
}
