import { createLogger } from '~/logger'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
  CompactDocument,
} from '~/isodb-core/types'
import { IPrimaryStorage } from './primary-storage'

const log = createLogger('isodb-primary')

function compactDocument(rev: number) {
  return <T extends IDocument>(item: T): CompactDocument<T> => {
    // return all items on initial request
    if (rev === 0) {
      return item
    }

    // then return only items with newer revision
    return (item._rev || 0) > rev ? item : item._id
  }
}

export default class PrimaryDB {
  constructor(public _storage: IPrimaryStorage) { }

  getRecords() {
    return this._storage.getRecords()
  }

  getAttachments() {
    return this._storage.getAttachments()
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
    // ensure client had latest revision
    if (this._storage.getRev() !== changeset.baseRev) {
      log.debug(`can't apply changeset: expected rev ${this._storage.getRev()}, got ${changeset.baseRev}`)

      return this._getChangesetResult(changeset.baseRev, false)
    }

    // skip empty changesets
    if (!changeset.records.length && !changeset.attachments.length) {
      log.debug('got empty changeset, skipping rev increase')

      return this._getChangesetResult(changeset.baseRev, true)
    }

    log.debug(`got ${changeset.records.length} records and ${changeset.attachments.length} attachments`)

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

  /**
   * @param rev minimum revision to include
   */
  private _getChangesetResult(rev = 0, success: boolean): IChangesetResult {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    const compact = compactDocument(rev)

    return {
      success,
      baseRev: rev,
      currentRev,
      records: this._storage.getRecords().map(compact),
      attachments: this._storage.getAttachments().map(compact),
    }
  }

}
