import path from 'path'
import { createLogger } from '~/logger'
import {
  getLastEl,
  prettyPrintJSON,
} from '~/utils'
import {
  listDirs,
  listFiles,
  readJSON,
  fileExists,
  FSTransaction,
  LockFile,
  dirExists,
  ensureDirExists,
} from '~/utils/fs'
import {
  IDocument,
  IAttachment,
} from '../../types'

type StorageUpdater<T extends IDocument> = (mutations: FSStorageMutations<T>) => Promise<void>

const log = createLogger('fs-storage')

export class FSStorage<T extends IDocument> {
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

  private acquireLock() {
    return this._lock.create()
  }

  private async init() {
    await Promise.all([
      ensureDirExists(this._documentsDir),
      ensureDirExists(this._attachmentsDir),
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

  static async create(rootDirRaw: string) {
    const rootDir = path.resolve(rootDirRaw)
    log.info(`arhiv root: ${rootDir}`)

    if (!await dirExists(rootDir)) {
      throw new Error(`${rootDir} doesn't exist`)
    }

    const storage = new FSStorage(rootDir)
    await storage.acquireLock()

    try {
      await storage.init()

      return storage
    } catch (e) {
      await storage.stop()
      throw e
    }
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

    if (!await dirExists(documentDir)) {
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
    const tx = new FSTransaction()
    const mutations = new FSStorageMutations(
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

class FSStorageMutations<T extends IDocument> {
  constructor(
    private _rev: number,
    private _documentsDir: string,
    private _attachmentsDir: string,
    private _tx: FSTransaction,
  ) { }

  setRev = (rev: number) => {
    this._rev = rev
  }

  getRev = () => this._rev

  putDocument = async (document: T) => {
    const documentDir = path.join(this._documentsDir, document._id)
    if (!await dirExists(documentDir)) {
      await this._tx.createDir(documentDir)
    }

    const filePath = path.join(documentDir, document._rev.toString())
    if (await fileExists(filePath)) {
      throw new Error(`document ${document._id} of rev ${document._rev} already exists`)
    }

    await this._tx.createFile(filePath, prettyPrintJSON(document))
  }

  addAttachment = async (attachment: IAttachment, attachmentPath: string) => {
    const attachmentDir = path.join(this._attachmentsDir, attachment._id)
    if (await dirExists(attachmentDir)) {
      throw new Error(`attachment ${attachment._id} already exists`)
    }

    await this._tx.createDir(attachmentDir)

    const metadataPath = path.join(attachmentDir, 'metadata')
    await this._tx.createFile(metadataPath, prettyPrintJSON(attachment))

    const dataPath = path.join(attachmentDir, 'data')
    await this._tx.moveFile(attachmentPath, dataPath)
  }

  updateAttachment = async (attachment: IAttachment) => {
    const dataPath = path.join(this._attachmentsDir, attachment._id, 'metadata')
    if (!await fileExists(dataPath)) {
      throw new Error(`attachment ${attachment._id} doesn't exist`)
    }

    await this._tx.updateFile(dataPath, prettyPrintJSON(attachment))
  }

  removeAttachmentData = async (id: string) => {
    const dataPath = path.join(this._attachmentsDir, id, 'data')
    await this._tx.deleteFile(dataPath)
  }
}
