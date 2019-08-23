import { IAttachment } from '~/isodb/types'
import { ArhivReplica } from '../types'

export class Attachment {
  constructor(
    private _replica: ArhivReplica,
    private _attachment: IAttachment,
  ) { }

  get url() {
    const url = this._replica.getAttachmentUrl(this.id).currentValue
    if (!url) {
      throw new Error(`can't get url for the attachment ${this.id}`)
    }

    return url
  }

  get id() {
    return this._attachment._id
  }
}
