import { createLogger } from '~/logger'
import {
  Dict,
} from '~/utils'
import { getMimeType } from '~/file-prober'
import { getFileSize } from '~/utils/fs'
import {
  IChangesetResponse,
  IChangeset,
  IDocument,
  ChangesetResponseStatus,
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

  getSchemaVersion() {
    return this._storage.getSchemaVersion()
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

  async applyChangeset(changeset: IChangeset<T>, attachedFiles: Dict): Promise<IChangesetResponse<T>> {
    const serverRev = this._storage.getRev()

    // this should never happen
    if (changeset.baseRev > serverRev) {
      throw new Error(`replica revision ${changeset.baseRev} is bigger than server revision ${serverRev}`)
    }

    const schemaVersion = this.getSchemaVersion()
    if (changeset.schemaVersion !== schemaVersion) {
      throw new Error(`replica schema version ${changeset.schemaVersion} is different than server schema version ${schemaVersion}`)
    }

    // on empty changeset just send latest changes to the replica
    if (isEmptyChangeset(changeset)) {
      log.debug('got empty changeset, skipping rev increase')

      return this._getChangesetResponse(changeset.baseRev, 'accepted')
    }

    // changeset isn't empty, but client isn't on latest revision
    if (changeset.baseRev < serverRev) {
      log.debug(`can't apply changeset: expected rev ${this._storage.getRev()}, got ${changeset.baseRev}`)

      return this._getChangesetResponse(changeset.baseRev, 'outdated')
    }

    log.debug(`got ${changeset.documents.length} documents and ${changeset.attachments.length} attachments`)

    await this._storage.updateStorage(async (mutations) => {
      const newRev = serverRev + 1

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

    return this._getChangesetResponse(changeset.baseRev, 'accepted')
  }

  /**
   * @param rev minimum revision to include
   */
  private async _getChangesetResponse(rev: number, status: ChangesetResponseStatus): Promise<IChangesetResponse<T>> {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    return {
      status,
      baseRev: rev,
      schemaVersion: this.getSchemaVersion(),
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
