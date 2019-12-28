import path from 'path'
import { createLogger } from '~/logger'
import {
  getLastEl,
  prettyPrintJSON,
  parseInt10,
} from '~/utils'
import {
  listDirs,
  listFiles,
  readJSON,
  fileExists,
  FSTransaction,
  LockFile,
  dirExists,
  assertDirExists,
  mkdir,
  writeJSON,
} from '~/utils/fs'
import {
  IDocument,
  IAttachment,
} from '../types'

type StorageUpdater<T extends IDocument> = (mutations: FSStorageMutations<T>, newRev: number) => Promise<void>

const log = createLogger('fs-storage')

interface IMetadata {
  schemaVersion: number
  revision: number
}

export class FSStorage<T extends IDocument> {
  public static readonly SCHEMA_VERSION = 1

  private _metadata: IMetadata = {
    schemaVersion: FSStorage.SCHEMA_VERSION,
    revision: 0,
  }

  private _documentsDir: string
  private _attachmentsDir: string
  private _metadataFile: string
  private _lock: LockFile

  private constructor(private _rootDir: string) {
    this._documentsDir = path.join(_rootDir, 'documents')
    this._attachmentsDir = path.join(_rootDir, 'attachments')
    this._metadataFile = path.join(_rootDir, 'metadata.json')
    this._lock = new LockFile(path.join(_rootDir, 'lock'))
    // TODO validate documents
    // check if dir name === document id
  }

  private async _init() {
    await Promise.all([
      assertDirExists(this._rootDir),
      assertDirExists(this._documentsDir),
      assertDirExists(this._attachmentsDir),
    ])

    this._metadata = await readJSON<IMetadata>(this._metadataFile)
    log.debug(`app schema version: ${FSStorage.SCHEMA_VERSION}`)
    log.debug(`data schema version: ${this._metadata.schemaVersion}`)

    if (this._metadata.schemaVersion !== FSStorage.SCHEMA_VERSION) {
      throw new Error(`app schema version is ${FSStorage.SCHEMA_VERSION}, data version is ${this._metadata.schemaVersion}`)
    }
  }

  private async _create() {
    await mkdir(this._rootDir)
    log.debug(`created dir ${this._rootDir}`)

    await mkdir(this._documentsDir)
    log.debug(`created dir ${this._documentsDir}`)

    await mkdir(this._attachmentsDir)
    log.debug(`created dir ${this._attachmentsDir}`)

    await this._writeMetadata()
    log.debug(`wrote metadata file ${this._metadataFile}`)
  }

  static async open(rootDirRaw: string, create = false) {
    const rootDir = path.resolve(rootDirRaw)
    log.info(`arhiv root: ${rootDir}`)

    const storage = new FSStorage(rootDir)
    if (!await dirExists(rootDir) && create) {
      log.info('initializing dir structure')
      await storage._create()
    }

    await storage._lock.create()

    try {
      await storage._init()

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
    return this._metadata.revision
  }

  getSchemaVersion() {
    return this._metadata.schemaVersion
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

    if (!await dirExists(documentDir, true)) {
      return undefined
    }

    const lastRev = getLastEl((await listFiles(documentDir)).map(parseInt10).sort())
    if (!lastRev) {
      throw new Error(`unreachable: document ${id} has no revisions`)
    }

    // FIXME check if looks like a document dir, check files etc

    return readJSON<T>(path.join(documentDir, lastRev.toString()))
  }

  async getDocumentHistory(id: string) {
    const documentDir = path.join(this._documentsDir, id)
    const revisions = (await listFiles(documentDir)).map(parseInt10).sort()

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
      this._documentsDir,
      this._attachmentsDir,
      tx,
    )

    try {
      const newRev = this.getRev() + 1
      await update(mutations, newRev)

      await tx.complete()

      this._metadata.revision = newRev
      await this._writeMetadata().catch((e) => {
        log.error('Failed to write metadata during storage update', e)
      })
    } catch (e) {
      await tx.revert()

      throw e
    }
  }

  async _writeMetadata() {
    await writeJSON(this._metadataFile, this._metadata)
  }
}

class FSStorageMutations<T extends IDocument> {
  constructor(
    private _documentsDir: string,
    private _attachmentsDir: string,
    private _tx: FSTransaction,
  ) { }

  putDocument = async (document: T) => {
    const documentDir = path.join(this._documentsDir, document._id)
    if (!await dirExists(documentDir, true)) {
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
    if (await dirExists(attachmentDir, true)) {
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
