import { createLogger } from '~/logger'
import {
  Dict,
} from '~/utils'
import { getMimeType } from '~/file-prober'
import { getFileSize } from '~/utils/fs'
import { isEmptyChangeset } from '../utils'
import { FSStorage } from './fs-storage'
import {
  IChangeset,
  IChangesetResponse,
  ChangesetResponseStatus,
} from '../types'

const log = createLogger('arhiv-db')

export class ArhivDB {
  constructor(private _storage: FSStorage) { }

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

  async applyChangeset(changeset: IChangeset, attachedFiles: Dict): Promise<IChangesetResponse> {
    const serverRev = this._storage.getRev()

    // this should never happen
    if (changeset.baseRev > serverRev) {
      throw new Error(`replica revision ${changeset.baseRev} is bigger than server revision ${serverRev}`)
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

    await this._storage.updateStorage(async (mutations, newRev) => {
      for (const changedDocument of changeset.documents) {
        await mutations.putDocument({
          ...changedDocument,
          rev: newRev,
        })
      }

      // update metadata and save new attachments
      await Promise.all(changeset.attachments.map(async (newAttachment) => {
        if (await this.getAttachment(newAttachment.id)) {
          throw new Error(`Attachment ${newAttachment.id} already exists`)
        }

        const attachedFile = attachedFiles[newAttachment.id]

        if (!attachedFile) {
          throw new Error(`File is missing for the new attachment ${newAttachment.id}`)
        }

        const [
          mimeType,
          size,
        ] = await Promise.all([
          getMimeType(attachedFile),
          getFileSize(attachedFile),
        ])

        await mutations.addAttachment({
          ...newAttachment,
          rev: newRev,
          mimeType,
          size,
        }, attachedFile)
      }))
    })

    return this._getChangesetResponse(changeset.baseRev, 'accepted')
  }

  /**
   * @param rev minimum revision to include
   */
  private async _getChangesetResponse(rev: number, status: ChangesetResponseStatus): Promise<IChangesetResponse> {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    return {
      status,
      baseRev: rev,
      currentRev,
      documents: (await this._storage.getDocuments()).filter(document => document.rev > rev),
      attachments: (await this._storage.getAttachments()).filter(attachment => attachment.rev > rev),
    }
  }

  private async _getUnusedAttachments() {
    const idsInUse = (await this._storage.getDocuments()).flatMap(document => document.attachmentRefs)

    return (await this._storage.getAttachments())
      .filter(attachment => !attachment.deleted && !idsInUse.includes(attachment.id))
  }

  /**
   * Remove unused attachments
   */
  async compact() {
    const unusedAttachments = await this._getUnusedAttachments()
    if (!unusedAttachments.length) {
      return
    }

    await this._storage.updateStorage(async (mutations, newRev) => {
      for (const attachment of unusedAttachments) {
        await mutations.updateAttachment({
          ...attachment,
          rev: newRev,
        })
        await mutations.removeAttachmentData(attachment.id)

        log.warn(`Removing unused attachment's data ${attachment.id}`)
      }
    })
  }
}
