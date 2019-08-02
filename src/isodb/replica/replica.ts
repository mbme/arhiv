import {
  PubSub,
  nowS,
} from '~/utils'
import { createLogger } from '~/logger'
import {
  IAttachment,
  IDocument,
  IChangesetResult,
  IChangeset,
} from '../types'
import { generateRandomId } from '../utils'
import {
  IReplicaStorage,
  LocalAttachments,
} from './replica-storage'
import { MergeConflicts } from './merge-conflict'

const logger = createLogger('isodb-replica')

export type Events = { name: 'db-update' } | { name: 'merge-conflicts' } | { name: 'merge-conflicts-resolved' }

export class IsodbReplica<T extends IDocument> {
  mergeConflicts?: MergeConflicts<T>

  constructor(
    private _storage: IReplicaStorage<T>,
    public events = new PubSub<Events>(),
  ) { }

  getRev() {
    return this._storage.getRev()
  }

  getAttachmentUrl(id: string) {
    return this._storage.getAttachmentUrl(id)
  }

  getDocument(id: string): T | undefined {
    return this._storage.getLocalDocument(id) || this._storage.getDocument(id)
  }

  getAttachment(id: string): IAttachment | undefined {
    return this._storage.getLocalAttachment(id) || this._storage.getAttachment(id)
  }

  getRandomId() {
    let id: string

    do {
      id = generateRandomId()
    } while (this.getDocument(id) || this.getAttachment(id)) // make sure generated id is free

    return id
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

  private _assertNoMergeConflicts() {
    if (this.mergeConflicts) {
      throw new Error('there is a pending merge conflict')
    }
  }

  saveAttachment(blob: File) {
    this._assertNoMergeConflicts()

    const id = this.getRandomId()
    this._storage.addLocalAttachment({
      _id: id,
      _rev: this.getRev(),
      _createdTs: nowS(),
    }, blob)
    // FIXME addLocalAttachment should save attachment to the long-term storage
    // only after saving document which references it
    logger.debug(`saved new attachment with id ${id}`)

    this.events.emit({ name: 'db-update' })

    return id
  }

  saveDocument(document: T) {
    this._assertNoMergeConflicts()

    this._storage.addLocalDocument({
      ...document,
      _updatedTs: nowS(),
    })
    logger.debug(`saved document with id ${document._id}`)

    this.events.emit({ name: 'db-update' })
  }

  getChangeset(): [IChangeset, LocalAttachments] {
    this._assertNoMergeConflicts()

    const changeset = {
      baseRev: this.getRev(),
      documents: this._storage.getLocalDocuments(),
      attachments: this._storage.getLocalAttachments(),
    }

    return [changeset, this._storage.getLocalAttachmentsData()]
  }

  async applyChangesetResult(changesetResult: IChangesetResult<T>) {
    this._assertNoMergeConflicts()

    if (this.getRev() !== changesetResult.baseRev) {
      throw new Error(`Got rev ${changesetResult.baseRev} instead of ${this.getRev()}`)
    }

    // TODO sync/locks
    // "success" means there should be no merge conflicts, so just update the data
    if (changesetResult.success) {
      this._storage.clearLocalData()

      this._storage.upgrade(
        changesetResult.currentRev,
        changesetResult.documents,
        changesetResult.attachments,
      )

      this.events.emit({ name: 'db-update' })

      return
    }

    this.mergeConflicts = new MergeConflicts((documents) => {
      // save resolved versions of the documents
      for (const document of documents) {
        this._storage.addLocalDocument(document)
      }

      this._storage.upgrade(
        changesetResult.currentRev,
        changesetResult.documents,
        changesetResult.attachments,
      )

      this.mergeConflicts = undefined

      this.events.emit({ name: 'db-update' })
      this.events.emit({ name: 'merge-conflicts-resolved' })
    })

    for (const localDocument of this._storage.getLocalDocuments()) {
      const remoteDocument = changesetResult.documents.find(document => document._id === localDocument._id)

      if (remoteDocument) {
        this.mergeConflicts.addConflict(
          this._storage.getDocument(localDocument._id)!,
          remoteDocument,
          localDocument,
        )
      }
    }

    this.events.emit({ name: 'merge-conflicts' })
  }
}
