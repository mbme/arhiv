import {
  createLogger,
  nowS,
} from '~/utils'
import { Cell } from '~/utils/reactive'
import {
  IAttachment,
  IDocument,
  IChangesetResult,
  IChangeset,
} from '../types'
import {
  IReplicaStorage,
  LocalAttachments,
} from './replica-storage'
import { MergeConflicts } from './merge-conflict'

const log = createLogger('isodb-replica')

export class IsodbReplica<T extends IDocument> {
  readonly updateTime$ = new Cell(0)
  readonly mergeConflicts$ = new Cell<MergeConflicts<T> | undefined>(undefined)

  constructor(
    private _storage: IReplicaStorage<T>,
  ) { }

  getRev() {
    return this._storage.getRev()
  }

  getLocalAttachmentData(id: string) {
    return this._storage.getLocalAttachmentData(id)
  }

  getDocument(id: string): T | undefined {
    return this._storage.getLocalDocument(id) || this._storage.getDocument(id)
  }

  getAttachment(id: string): IAttachment | undefined {
    return this._storage.getLocalAttachment(id) || this._storage.getAttachment(id)
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

  hasMergeConflicts() {
    return !!this.mergeConflicts$.value
  }

  private _assertNoMergeConflicts() {
    if (this.hasMergeConflicts()) {
      throw new Error('there is a pending merge conflict')
    }
  }

  private _onUpdate() {
    this.updateTime$.value = nowS()
  }

  saveAttachment(id: string, blob: File) {
    this._assertNoMergeConflicts()

    this._storage.addLocalAttachment({
      _id: id,
      _rev: this.getRev(),
      _createdTs: nowS(),
      _mimeType: blob.type,
      _size: blob.size,
    }, blob)
    log.debug(`saved new attachment with id ${id}`)

    this._onUpdate()
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

  getChangeset(): [IChangeset<T>, LocalAttachments] {
    this._assertNoMergeConflicts()

    return this._storage.getChangeset()
  }

  async applyChangesetResult(changesetResult: IChangesetResult<T>) {
    this._assertNoMergeConflicts()

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

      this.mergeConflicts$.value = undefined
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
      this.mergeConflicts$.value = mergeConflicts
    } else {
      this._storage.upgrade(changesetResult)
      this._onUpdate()
    }
  }

  /**
   * Remove unused local attachments
   */
  compact() {
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
}
