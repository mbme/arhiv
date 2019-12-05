import { createLogger } from '~/logger'
import {
  Dict,
} from '~/utils'
import { getMimeType } from '~/file-prober'
import { getFileSize } from '~/utils/fs'
import {
  IChangesetResult,
  IChangeset,
  IDocument,
} from '../../types'
import { isEmptyChangeset } from '../../utils'
import { FSStorage } from './fs-storage'

const log = createLogger('arhiv-db')

export class ArhivDB<T extends IDocument> {
  constructor(private _storage: FSStorage<T>) { }

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

  getDocumentHistory(id: string) {
    return this._storage.getDocumentHistory(id)
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

  async applyChangeset(changeset: IChangeset<T>, attachedFiles: Dict): Promise<IChangesetResult<T>> {
    const baseRev = this._storage.getRev()

    // this should never happen
    if (changeset.baseRev > baseRev) {
      throw new Error(`got replica revision ${changeset.baseRev} bigger than server revision ${baseRev}`)
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

    await this._storage.updateStorage(async (mutations) => {
      const newRev = baseRev + 1

      for (const changedDocument of changeset.documents) {
        await mutations.putDocument({
          ...changedDocument,
          _rev: newRev,
        })
      }

      // update metadata and save new attachments
      await Promise.all(changeset.attachments.map(async (newAttachment) => {
        if (await this.getAttachment(newAttachment._id)) {
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

        await mutations.addAttachment({
          ...newAttachment,
          _rev: newRev,
          _mimeType,
          _size,
        }, attachedFile)
      }))

      mutations.setRev(newRev)
    })

    return this._getChangesetResult(changeset.baseRev, true)
  }

  /**
   * @param rev minimum revision to include
   */
  private async _getChangesetResult(rev: number, success: boolean): Promise<IChangesetResult<T>> {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    return {
      success,
      baseRev: rev,
      currentRev,
      documents: (await this._storage.getDocuments()).filter(document => document._rev > rev),
      attachments: (await this._storage.getAttachments()).filter(attachment => attachment._rev > rev),
    }
  }

  private async _getUnusedAttachments() {
    const idsInUse = (await this._storage.getDocuments()).flatMap(document => document._attachmentRefs)

    return (await this._storage.getAttachments())
      .filter(attachment => !attachment._deleted && !idsInUse.includes(attachment._id))
  }

  /**
   * Remove unused attachments
   */
  async compact() {
    const unusedAttachments = await this._getUnusedAttachments()
    if (!unusedAttachments.length) {
      return
    }

    await this._storage.updateStorage(async (mutations) => {
      const newRev = this.getRev() + 1

      for (const attachment of unusedAttachments) {
        await mutations.updateAttachment({
          ...attachment,
          _rev: newRev,
        })
        await mutations.removeAttachmentData(attachment._id)

        log.warn(`Removing unused attachment's data ${attachment._id}`)
      }

      mutations.setRev(newRev)
    })
  }
}
