import {
  IChangesetResult,
  IChangeset,
} from '~/isodb-core/types'
import { IPrimaryStorage } from './primary-storage'

export default class PrimaryDB {
  constructor(public _storage: IPrimaryStorage) { }

  getRecords() {
    return this._storage.getRecords()
  }

  getAttachments() {
    return this._storage.getAttachments()
  }

  /**
   * @param rev minimum revision to include
   */
  _getChangesetResult(rev = 0, success: boolean): IChangesetResult {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    return {
      success,
      baseRev: rev,
      currentRev,
      records: this._storage.getRecords().map(item => item._rev! >= rev ? item : item._id),
      attachments: this._storage.getAttachments().map(item => item._rev! >= rev ? item : item._id),
    }
  }

  /**
   * @returns storage revision
   */
  getRev() {
    return this._storage.getRev()
  }

  /**
   * @param id record id
   */
  getRecord(id: string) {
    return this._storage.getRecord(id)
  }

  /**
   * @param id attachment id
   */
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

  applyChangeset(changeset: IChangeset, attachedFiles: { [id: string]: string }) {
    if (this._storage.getRev() !== changeset.baseRev) { // ensure client had latest revision
      return this._getChangesetResult(changeset.baseRev, false)
    }

    if (!changeset.records.length && !changeset.attachments.length) { // skip empty changesets
      return this._getChangesetResult(changeset.baseRev, true)
    }

    const newRev = changeset.baseRev + 1

    for (const changedRecord of changeset.records) {
      this._storage.putRecord({
        ...changedRecord,
        _rev: newRev,
      })
    }

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
        })
      } else {
        this._storage.addAttachment({
          ...changedAttachment,
          _rev: newRev,
        }, attachedFile)
      }
    }

    this._storage.setRev(newRev)

    return this._getChangesetResult(changeset.baseRev, true)
  }
}
