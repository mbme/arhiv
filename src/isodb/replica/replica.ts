import {
  isString,
  array2object,
  PubSub,
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

const logger = createLogger('isodb-replica')

export interface IEvents {
  'db-update': undefined
}

export class IsodbReplica {
  constructor(
    private _storage: IReplicaStorage,
    public events = new PubSub<IEvents>(),
  ) { }

  private _notify() {
    this.events.emit('db-update', undefined)
  }

  getRev() {
    return this._storage.getRev()
  }

  getAttachmentUrl(id: string) {
    return this._storage.getAttachmentUrl(id)
  }

  getDocument(id: string): IDocument | undefined {
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

  getDocuments(): IDocument[] {
    const localDocuments = this._storage.getLocalDocuments()
    const localIds = new Set(localDocuments.map(item => item._id))

    const documents = this._storage.getDocuments().filter(item => !localIds.has(item._id))

    // TODO sort
    return [
      ...documents,
      ...localDocuments,
    ].filter(document => !document._deleted) // FIXME what to do with deleted documents?
  }

  saveAttachment(attachment: IAttachment, blob?: File) {
    if (!this.getAttachment(attachment._id) && !blob) {
      throw new Error(`new attachment ${attachment._id}: blob missing`)
    }

    this._storage.addLocalAttachment(attachment, blob)

    this._notify()
  }

  saveDocument(document: IDocument) {
    this._storage.addLocalDocument(document)

    this.compact()

    this._notify()
  }

  getChangeset(): [IChangeset, LocalAttachments] {
    const changeset = {
      baseRev: this.getRev(),
      documents: this._storage.getLocalDocuments(),
      attachments: this._storage.getLocalAttachments(),
    }

    return [changeset, this._storage.getLocalAttachmentsData()]
  }

  async applyChangesetResult(changesetResult: IChangesetResult) {
    if (this.getRev() !== changesetResult.baseRev) {
      throw new Error(`Got rev ${changesetResult.baseRev} instead of ${this.getRev()}`)
    }

    const currentDocuments = array2object(this._storage.getDocuments(), item => item._id)
    const newDocuments = changesetResult.documents.map(item => isString(item) ? currentDocuments[item] : item)

    const currentAttachments = array2object(this._storage.getAttachments(), item => item._id)
    const newAttachments = changesetResult.attachments.map(item => isString(item) ? currentAttachments[item] : item)

    if (changesetResult.success) {
      this._storage.upgrade(changesetResult.currentRev, newDocuments, newAttachments)
      this._storage.clearLocalData()
      this._notify()

      return false
    }

    const documentConflicts = []
    // for each local document
    for (const localDocument of this._storage.getLocalDocuments()) {
      const existingDocument = currentDocuments[localDocument._id]
      const newDocument = newDocuments.find(item => item._id === localDocument._id)!

      // if is existing document & revision changed
      //   mark as a conflict
      if (existingDocument._rev !== newDocument._rev) {
        documentConflicts.push({
          base: existingDocument,
          updated: newDocument,
          local: localDocument,
        })
      }
    }
    const attachmentConflicts = [] // for each local attachment
    for (const localAttachment of this._storage.getLocalAttachments()) {
      const existingAttachment = currentAttachments[localAttachment._id]
      const newAttachment = newAttachments.find(item => item._id === localAttachment._id)!

      // if is existing attachment & revision changed
      //   mark as a conflict
      if (existingAttachment._rev !== newAttachment._rev) {
        attachmentConflicts.push({
          base: existingAttachment,
          updated: newAttachment,
          local: localAttachment,
        })
      }
    }

    // resolve conflicts if needed
    if (documentConflicts.length || attachmentConflicts.length) {
      const resolvedConflicts = await merge({ documents: documentConflicts, attachments: attachmentConflicts })

      for (const updatedDocument of resolvedConflicts.documents) {
        this._storage.addLocalDocument(updatedDocument)
      }

      for (const updatedAttachment of resolvedConflicts.attachments) {
        this._storage.addLocalAttachment(updatedAttachment)
      }
    }

    // for each local document
    //   if references deleted document
    //     restore deleted document & all deleted documents referenced by it
    const idsToCheck = this._storage.getLocalDocuments().flatMap(item => item._refs)
    const idsChecked = new Set()
    while (idsToCheck.length) {
      const id = idsToCheck.shift()!

      if (idsChecked.has(id)) continue

      const existingDocument = currentDocuments[id]
      const newDocument = newDocuments.find(item => item._id === id)
      if (existingDocument && !newDocument) {
        logger.info(`Restoring document ${id}`)
        this._storage.addLocalDocument(existingDocument) // restore document
        idsToCheck.push(...existingDocument._refs)
      }

      idsChecked.add(id)
    }

    this._storage.upgrade(changesetResult.currentRev, newDocuments, newAttachments)

    this._notify()

    return true
  }

  /**
   * Remove unused local attachments
   */
  compact() {
    const idsInUse = new Set()
    for (const document of this._storage.getDocuments()) {
      for (const id of document._attachmentRefs) {
        idsInUse.add(id)
      }
    }
    const localAttachmentIds = new Set(this._storage.getLocalAttachments().map(item => item._id))

    for (const id of localAttachmentIds) {
      // remove unused new local attachments
      if (!idsInUse.has(id)) {
        logger.warn(`Removing unused local attachment ${id}`)
        this._storage.removeLocalAttachment(id)
      }
    }
  }
}
