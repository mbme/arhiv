import path from 'path'
import {
  getLastEl,
} from '~/utils'
import {
  listDirs,
  listFiles,
  readJSON,
  fileExists,
  mkdir,
  FSTransaction,
  LockFile,
} from '~/utils/fs'
import {
  IDocument,
  IAttachment,
} from '../types'
import {
  IPrimaryStorage,
  StorageUpdater,
} from './primary-storage'
import {
  PrimaryFSStorageMutations,
} from './primary-fs-storage-mutations'

export class PrimaryFSStorage<T extends IDocument> implements IPrimaryStorage<T> {
  private _rev = 0

  private _documentsDir: string
  private _attachmentsDir: string
  private _lock: LockFile

  private constructor(rootDir: string) {
    this._documentsDir = path.join(rootDir, 'documents')
    this._attachmentsDir = path.join(rootDir, 'attachments')
    this._lock = new LockFile(path.join(rootDir, 'lock'))
    // TODO validate documents
    // check if dir name === document id
  }

  private async init() {
    await Promise.all([
      this._lock.create(),
      mkdir(this._documentsDir),
      mkdir(this._attachmentsDir),
    ])

    const [
      documents,
      attachments,
    ] = await Promise.all([
      this.getDocuments(),
      this.getAttachments(),
    ])

    const maxDocumentRev = documents.reduce((acc, document) => Math.max(acc, document._rev), 0)
    const maxAttachmentRev = attachments.reduce((acc, attachment) => Math.max(acc, attachment._rev), 0)

    this._rev = Math.max(maxDocumentRev, maxAttachmentRev)
  }

  static async create(rootDir: string) {
    const storage = new PrimaryFSStorage(rootDir)

    await storage.init()

    return storage
  }

  async stop() {
    await this._lock.destroy()
  }

  getRev() {
    return this._rev
  }

  async getDocuments() {
    const dirs = await listDirs(this._documentsDir)

    // FIXME check if looks like a document dir, check files etc
    return Promise.all(dirs.map(async (dir) => {
      const document = await this.getDocument(dir)

      if (!document) {
        throw new Error(`unreachable: expected document ${dir} doesn't exist`)
      }

      return document
    }))
  }

  async getDocument(id: string) {
    const documentDir = path.join(this._documentsDir, id)

    if (!await fileExists(documentDir)) {
      return undefined
    }

    const lastRev = getLastEl((await listFiles(documentDir)).map(parseInt).sort())

    // FIXME check if looks like a document dir, check files etc

    return readJSON<T>(path.join(documentDir, lastRev.toString()))
  }

  async getDocumentHistory(id: string) {
    const documentDir = path.join(this._documentsDir, id)
    const revisions = (await listFiles(documentDir)).map(parseInt).sort()

    return Promise.all(revisions.map(revision => readJSON<T>(path.join(documentDir, revision.toString()))))
  }

  async getAttachments() {
    const dirs = await listDirs(this._attachmentsDir)

    // FIXME check if looks like a document dir, check files etc
    return Promise.all(dirs.map(async (dir) => {
      const attachment = await this.getAttachment(dir)

      if (!attachment) {
        throw new Error(`unreachable: expected attachment ${dir} doesn't exist`)
      }

      return attachment
    }))
  }

  async getAttachment(id: string) {
    const attachmentFile = path.join(this._attachmentsDir, id, 'metadata')

    if (!await fileExists(attachmentFile)) {
      return undefined
    }

    return readJSON<IAttachment>(attachmentFile)
  }

  async getAttachmentDataPath(id: string) {
    const attachmentDataPath = path.join(this._attachmentsDir, id, 'data')

    if (!await fileExists(attachmentDataPath)) {
      return undefined
    }

    return attachmentDataPath
  }

  async updateStorage(update: StorageUpdater<T>) {
    const tx = await FSTransaction.create()
    const mutations = new PrimaryFSStorageMutations(
      this._rev,
      this._documentsDir,
      this._attachmentsDir,
      tx,
    )

    try {
      await update(mutations)

      await tx.complete()

      this._rev = mutations.getRev()
    } catch (e) {
      await tx.revert()

      throw e
    }
  }
}
