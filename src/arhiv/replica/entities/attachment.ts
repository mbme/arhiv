import { blobUrl$ } from '~/reactive'
import { IAttachment } from '../../types'
import { ArhivDB } from '../db'

export class Attachment {
  constructor(
    private _db: ArhivDB,
    public attachment: IAttachment,
  ) { }

  getUrl$() {
    return this._db.getAttachmentData$(this.id)
      .take(1)
      .switchMap(blobUrl$)
  }

  get id() {
    return this.attachment._id
  }
}
