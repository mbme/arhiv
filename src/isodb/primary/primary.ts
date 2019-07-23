import { createLogger } from '~/logger'
import { getMimeType } from '~/file-prober'
import { getFileSize } from '~/utils/fs'
import { IDict } from '~/utils'
import {
  IChangesetResult,
  IChangeset,
} from '~/isodb/types'
import { isEmptyChangeset } from '~/isodb/utils'
import { IPrimaryStorage } from './primary-storage'

const log = createLogger('isodb-primary')

export class PrimaryDB {
  constructor(private _storage: IPrimaryStorage) { }

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
  getAttachmentPath(id: string) {
    return this._storage.getAttachmentPath(id)
  }

  async applyChangeset(changeset: IChangeset, attachedFiles: IDict): Promise<IChangesetResult> {
    // ensure client had latest revision
    if (this._storage.getRev() !== changeset.baseRev) {
      log.debug(`can't apply changeset: expected rev ${this._storage.getRev()}, got ${changeset.baseRev}`)

      return this._getChangesetResult(changeset.baseRev, false)
    }

    // skip empty changesets
    if (isEmptyChangeset(changeset)) {
      log.debug('got empty changeset, skipping rev increase')

      return this._getChangesetResult(changeset.baseRev, true)
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
    for (const changedAttachment of changeset.attachments) {
      const existingAttachment = this.getAttachment(changedAttachment._id)
      const attachedFile = attachedFiles[changedAttachment._id]

      if (!existingAttachment && !attachedFile) {
        throw new Error(`File is missing for the new attachment ${changedAttachment._id}`)
      }
      if (existingAttachment && attachedFile) {
        throw new Error(`Can't update file for the attachment ${changedAttachment._id}`)
      }

      if (existingAttachment) {
        this._storage.updateAttachment({
          ...changedAttachment,
          _rev: newRev,
          _type: existingAttachment._type,
          _size: existingAttachment._size,
        })
      } else {
        const [
          _type,
          _size,
        ] = await Promise.all([
          await getMimeType(attachedFile),
          await getFileSize(attachedFile),
        ])

        this._storage.addAttachment({
          ...changedAttachment,
          _rev: newRev,
          _type,
          _size,
        }, attachedFile)
      }
    }

    this._storage.setRev(newRev)

    return this._getChangesetResult(changeset.baseRev, true)
  }

  /**
   * @param rev minimum revision to include
   */
  private _getChangesetResult(rev: number, success: boolean): IChangesetResult {
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

}
