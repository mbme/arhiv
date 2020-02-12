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
  IDocument,
  IAttachment,
} from '../types'

export class FSStorageMutations {
  constructor(
    private _documentsDir: string,
    private _attachmentsDir: string,
    private _tx: FSTransaction,
  ) { }

  putDocument = async (document: IDocument) => {
    const documentDir = path.join(this._documentsDir, document.id)
    if (!await dirExists(documentDir, true)) {
      await this._tx.createDir(documentDir)
    }

    const filePath = path.join(documentDir, document.rev.toString())
    if (await fileExists(filePath)) {
      throw new Error(`document ${document.id} of rev ${document.rev} already exists`)
    }

    await this._tx.createFile(filePath, prettyPrintJSON(document))
  }

  addAttachment = async (attachment: IAttachment, attachmentPath: string) => {
    const attachmentDir = path.join(this._attachmentsDir, attachment.id)
    if (await dirExists(attachmentDir, true)) {
      throw new Error(`attachment ${attachment.id} already exists`)
    }

    await this._tx.createDir(attachmentDir)

    const metadataPath = path.join(attachmentDir, 'metadata')
    await this._tx.createFile(metadataPath, prettyPrintJSON(attachment))

    const dataPath = path.join(attachmentDir, 'data')
    await this._tx.moveFile(attachmentPath, dataPath)
  }

  updateAttachment = async (attachment: IAttachment) => {
    const dataPath = path.join(this._attachmentsDir, attachment.id, 'metadata')
    if (!await fileExists(dataPath)) {
      throw new Error(`attachment ${attachment.id} doesn't exist`)
    }

    await this._tx.updateFile(dataPath, prettyPrintJSON(attachment))
  }

  removeAttachmentData = async (id: string) => {
    const dataPath = path.join(this._attachmentsDir, id, 'data')
    await this._tx.deleteFile(dataPath)
  }
}
