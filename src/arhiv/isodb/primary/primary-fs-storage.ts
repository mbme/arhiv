import path from 'path'
import {
  getLastEl,
} from '~/utils'
import {
  moveFile,
  listDirs,
  listFiles,
  readJSON,
  fileExists,
  mkdir,
  writeJSON,
  removeFile,
} from '~/utils/fs'
import { createFsTransaction } from '~/utils/fs-transaction';
import {
  IDocument,
  IAttachment,
} from '../types'
import {
  IPrimaryStorage,
  StorageUpdater,
  IPrimaryStorageMutations,
} from './primary-storage'

class PrimaryFSStorageMutations<T extends IDocument> implements IPrimaryStorageMutations<T> {
  private _transaction = createFsTransaction()

  constructor(
    private _rev: number,
    private _documentsDir: string,
    private _attachmentsDir: string,
  ) { }

  setRev = (rev: number) => {
    this._rev = rev
  }

  getRev = () => this._rev

  putDocument = async (document: T) => {
    const documentDir = path.join(this._documentsDir, document._id)
    if (!await fileExists(documentDir)) {
      await mkdir(documentDir)
    }

    const filePath = path.join(documentDir, document._rev.toString())
    if (await fileExists(filePath)) {
      throw new Error(`document ${document._id} of rev ${document._rev} already exists`)
    }

    await writeJSON(filePath, document)
  }

  addAttachment = async (attachment: IAttachment, attachmentPath: string) => {
    const attachmentDir = path.join(this._attachmentsDir, attachment._id)
    if (await fileExists(attachmentDir)) {
      throw new Error(`attachment ${attachment._id} already exists`)
    }

    await mkdir(attachmentDir)

    const metadataPath = path.join(attachmentDir, 'metadata')
    await writeJSON(metadataPath, attachment)

    const dataPath = path.join(attachmentDir, 'data')
    await moveFile(attachmentPath, dataPath)
  }

  updateAttachment = async (attachment: IAttachment) => {
    const dataPath = path.join(this._attachmentsDir, attachment._id, 'metadata')
    if (!await fileExists(dataPath)) {
      throw new Error(`attachment ${attachment._id} doesn't exist`)
    }

    await writeJSON(dataPath, attachment)
  }

  removeAttachmentData = async (id: string) => {
    const dataPath = path.join(this._attachmentsDir, id, 'data')
    await removeFile(dataPath)
  }
}

export class PrimaryFSStorage<T extends IDocument> implements IPrimaryStorage<T> {
  private _rev = 0

  private _documentsDir: string
  private _attachmentsDir: string

  private constructor(rootDir: string) {
    this._documentsDir = path.join(rootDir, 'documents')
    this._attachmentsDir = path.join(rootDir, 'attachments')
    // TODO validate documents, check if dirs exist, create if needed
    // check if file name === document id
  }

  private async init() {
    await Promise.all([
      mkdir(this._documentsDir),
      mkdir(this._attachmentsDir),
    ])

    const maxDocumentRev = (
      await this.getDocuments()
    ).reduce((acc, document) => Math.max(acc, document._rev), 0)
    const maxAttachmentRev = (
      await this.getAttachments()
    ).reduce((acc, attachment) => Math.max(acc, attachment._rev), 0)
    this._rev = Math.max(maxDocumentRev, maxAttachmentRev)
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
        throw new Error("unreachable: expected document doesn't exist")
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
    const dirs = await listDirs(this._documentsDir)

    // FIXME check if looks like a document dir, check files etc
    return Promise.all(dirs.map(async (dir) => {
      const attachment = await this.getAttachment(dir)

      if (!attachment) {
        throw new Error("unreachable: expected attachment doesn't exist")
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
    const mutations = new PrimaryFSStorageMutations(this._rev, this._documentsDir, this._attachmentsDir)

    await update(mutations)

    this._rev = mutations.getRev()
  }

  static async create(rootDir: string) {
    const storage = new PrimaryFSStorage(rootDir)

    await storage.init()

    return storage
  }
}
