import { IAttachment } from '~/isodb/types'
import { ArhivReplica } from '../types'

export class Attachment {
  constructor(
    private _replica: ArhivReplica,
    public attachment: IAttachment,
  ) { }

  get url() {
    const url = this._replica.getAttachmentUrl(this.id)
    if (!url) {
      throw new Error(`can't get url for the attachment ${this.id}`)
    }

    return url
  }

  get id() {
    return this.attachment._id
  }
}
