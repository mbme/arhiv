import { blobUrl$ } from '~/reactive'
import { IAttachment } from '~/arhiv/types'
import { ReplicaDB } from '../db'

export class AttachmentManager {
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
