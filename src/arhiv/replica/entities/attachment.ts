import { blobUrl$ } from '~/turbo'
import { IAttachment } from '~/arhiv/types'
import { ReplicaDB } from '../db'

export class Attachment {
  constructor(
    private _db: ReplicaDB,
    public attachment: IAttachment,
  ) { }

  getUrl$() {
    return this._db.getAttachmentData$(this.id)
      .take(1)
      .switchMap(blobUrl$)
  }

  get id() {
    return this.attachment.id
  }
}
