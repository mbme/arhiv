import { blobUrl$ } from '~/utils/reactive'
import { IAttachment } from '~/isodb/types'
import { ArhivReplica } from '../types'

export class Attachment {
  constructor(
    private _replica: ArhivReplica,
    public attachment: IAttachment,
  ) { }

  getUrl$() {
    return this._replica.getAttachmentData$(this.id)
      .filter(blob => !!blob)
      .take(1)
      .switchMap(blobUrl$)
  }

  get id() {
    return this.attachment._id
  }
}
