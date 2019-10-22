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
import {
  IDocument,
  IAttachment,
} from '../types'
import {
  IPrimaryStorage,
  StorageUpdater,
} from './primary-storage'

export class PrimaryFSStorage<T extends IDocument> implements IPrimaryStorage<T> {
  private _rev = 0

  private _documentsDir: string
  private _attachmentsDir: string

  constructor(rootDir: string) {
    this._documentsDir = path.join(rootDir, 'documents')
    this._attachmentsDir = path.join(rootDir, 'attachments')
    // TODO validate documents, check if dirs exist, create if needed
    // check if file name === document id
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

  private _setRev = (rev: number) => {
    this._rev = rev
  }

  private _putDocument = async (document: T) => {
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

  private _addAttachment = async (attachment: IAttachment, attachmentPath: string) => {
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

  private _updateAttachment = async (attachment: IAttachment) => {
    const dataPath = path.join(this._attachmentsDir, attachment._id, 'metadata')
    if (!await fileExists(dataPath)) {
      throw new Error(`attachment ${attachment._id} doesn't exist`)
    }

    await writeJSON(dataPath, attachment)
  }

  private _removeAttachmentData = async (id: string) => {
    const dataPath = path.join(this._attachmentsDir, id, 'data')
    await removeFile(dataPath)
  }

  async updateStorage(update: StorageUpdater<T>) {
    return update({
      setRev: this._setRev,
      putDocument: this._putDocument,
      addAttachment: this._addAttachment,
      updateAttachment: this._updateAttachment,
      removeAttachmentData: this._removeAttachmentData,
    })
  }
}
