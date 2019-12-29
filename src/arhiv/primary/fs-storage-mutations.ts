import path from 'path'
import {
  prettyPrintJSON,
} from '~/utils'
import {
  fileExists,
  FSTransaction,
  dirExists,
} from '~/utils/fs'
import {
  IAttachment,
  ArhivDocument,
} from '../types'

export class FSStorageMutations {
  constructor(
    private _documentsDir: string,
    private _attachmentsDir: string,
    private _tx: FSTransaction,
  ) { }

  putDocument = async (document: ArhivDocument) => {
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
