import {
  createLogger,
  IDict,
} from '~/utils'
import { getMimeType } from '~/file-prober'
import { getFileSize } from '~/utils/fs'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
} from '../types'
import { isEmptyChangeset } from '../utils'
import { IPrimaryStorage } from './primary-storage'

const log = createLogger('isodb-primary')

export class PrimaryDB<T extends IDocument> {
  constructor(private _storage: IPrimaryStorage<T>) { }

  getDocuments() {
    return this._storage.getDocuments()
  }

  getAttachments() {
    return this._storage.getAttachments()
  }

  getRev() {
    return this._storage.getRev()
  }

  getDocument(id: string) {
    return this._storage.getDocument(id)
  }

  getAttachment(id: string) {
    return this._storage.getAttachment(id)
  }

  /**
   * @param id attachment id
   * @returns path to attachment
   */
  getAttachmentDataPath(id: string) {
    return this._storage.getAttachmentDataPath(id)
  }

  async applyChangeset(changeset: IChangeset<T>, attachedFiles: IDict): Promise<IChangesetResult<T>> {
    const baseRev = this._storage.getRev()

    // this should never happen
    if (changeset.baseRev > baseRev) {
      throw new Error(`got replica revision ${changeset.baseRev} bigger than primary revision ${baseRev}`)
    }

    // on empty changeset just send latest changes to the replica
    if (isEmptyChangeset(changeset)) {
      log.debug('got empty changeset, skipping rev increase')

      return this._getChangesetResult(changeset.baseRev, true)
    }

    // ensure client had latest revision
    if (baseRev < changeset.baseRev) {
      log.debug(`can't apply changeset: expected rev ${this._storage.getRev()}, got ${changeset.baseRev}`)

      return this._getChangesetResult(changeset.baseRev, false)
    }

    log.debug(`got ${changeset.documents.length} documents and ${changeset.attachments.length} attachments`)

    const newRev = changeset.baseRev + 1

    for (const changedDocument of changeset.documents) {
      this._storage.putDocument({
        ...changedDocument,
        _rev: newRev,
      })
    }

    // TODO parallel this
    for (const newAttachment of changeset.attachments) {
      if (this.getAttachment(newAttachment._id)) {
        throw new Error(`Attachment ${newAttachment._id} already exists`)
      }

      const attachedFile = attachedFiles[newAttachment._id]

      if (!attachedFile) {
        throw new Error(`File is missing for the new attachment ${newAttachment._id}`)
      }

      const [
        _mimeType,
        _size,
      ] = await Promise.all([
        getMimeType(attachedFile),
        getFileSize(attachedFile),
      ])

      await this._storage.addAttachment({
        ...newAttachment,
        _rev: newRev,
        _mimeType,
        _size,
      }, attachedFile)
    }

    this._storage.setRev(newRev)

    return this._getChangesetResult(changeset.baseRev, true)
  }

  /**
   * @param rev minimum revision to include
   */
  private _getChangesetResult(rev: number, success: boolean): IChangesetResult<T> {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    return {
      success,
      baseRev: rev,
      currentRev,
      documents: this._storage.getDocuments().filter(document => document._rev > rev),
      attachments: this._storage.getAttachments().filter(attachment => attachment._rev > rev),
    }
  }

  private _getUnusedAttachments() {
    const idsInUse = this._storage.getDocuments().flatMap(document => document._attachmentRefs)

    return this._storage.getAttachments()
      .filter(attachment => !attachment._deleted && !idsInUse.includes(attachment._id))
  }

  /**
   * Remove unused attachments
   */
  compact() {
    const unusedAttachments = this._getUnusedAttachments()
    if (!unusedAttachments.length) {
      return
    }

    const newRev = this.getRev() + 1

    for (const attachment of unusedAttachments) {
      this._storage.updateAttachment({
        ...attachment,
        _rev: newRev,
      })
      this._storage.removeAttachmentData(attachment._id)

      log.warn(`Removing unused attachment's data ${attachment._id}`)
    }

    this._storage.setRev(newRev)
  }
}
