import { createLogger } from '~/logger'
import { PubSub } from '~/utils'
import { ReactiveValue } from '~/utils/reactive'
import { LockManager } from './lock-manager'
import {
  IsodbReplica,
  Events,
} from './replica'
import {
  IDocument,
  IChangeset,
  IChangesetResult,
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

type SyncState = 'sync' | 'merge-conflicts' | 'synced' | 'not-synced'

type ChangesetExchange<T extends IDocument> = (
  changeset: IChangeset<T>,
  blobs: LocalAttachments,
) => Promise<IChangesetResult<T>>

export class ReplicaManager<T extends IDocument> {
  events = new PubSub<Events>()
  locks = new LockManager()
  private _replica: IsodbReplica<T>

  $syncState: ReactiveValue<SyncState>
  $updateTime: ReactiveValue<number>

  constructor(storage: IReplicaStorage<T>) {
    this._replica = new IsodbReplica(storage, this.events)

    this.$syncState = new ReactiveValue<SyncState>('not-synced', (next) => {
      const onMergeConflicts = () => next('merge-conflicts')
      const onMergeConflictsResolved = () => next('not-synced')

      this.events.on('merge-conflicts', onMergeConflicts)
      this.events.on('merge-conflicts-resolved', onMergeConflictsResolved)

      return () => {
        this.events.off('merge-conflicts', onMergeConflicts)
        this.events.off('merge-conflicts-resolved', onMergeConflictsResolved)
      }
    })

    this.$updateTime = new ReactiveValue(0, (next) => {
      const onUpdate = () => next(Date.now())

      this.events.on('db-update', onUpdate)

      return () => {
        this.events.off('db-update', onUpdate)
      }
    })

    // run compaction if db isn't locked
    this.locks.state.subscribe((lockState) => {
      if (lockState === 'free') {
        this._replica.compact()
      }
    })
  }

  getRandomId() {
    let id: string

    do {
      id = generateRandomId()
    } while (this.getDocument(id) || this.getAttachment(id)) // make sure generated id is free

    return id
  }

  getAttachment(id: string) {
    return this._replica.getAttachment(id)
  }

  getAttachmentUrl(id: string) {
    return this._replica.getAttachmentUrl(id)
  }

  saveAttachment(file: File) {
    const id = this.getRandomId()
    this._replica.saveAttachment(id, file)

    return id
  }

  getDocuments() {
    return this._replica.getDocuments()
  }

  getDocument(id: string) {
    return this._replica.getDocument(id)
  }

  saveDocument(document: T) {
    this._replica.saveDocument(document)
  }

  async sync(exchange: ChangesetExchange<T>): Promise<boolean> {
    if (!this.locks.isFree()) {
      log.debug('Skipping sync: lock is not free')

      return false
    }

    if (this._replica.mergeConflicts) {
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
      log.info(`sync: succes=${result.success}`)

      await this._replica.applyChangesetResult(result)

      if (this._replica.mergeConflicts) {
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
    this.events.destroy()
    this.$syncState.destroy()
    this.$updateTime.destroy()
    // TODO stop storage?
  }
}
