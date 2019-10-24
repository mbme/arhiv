import path from 'path'
import {
  moveFile,
  fileExists,
  mkdir,
  writeJSON,
  removeFile,
} from '~/utils/fs'
import {
  FSTransaction,
} from '~/utils/fs-transaction';
import {
  IDocument,
  IAttachment,
} from '../types'
import {
  IPrimaryStorageMutations,
} from './primary-storage'

export class PrimaryFSStorageMutations<T extends IDocument> implements IPrimaryStorageMutations<T> {
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
