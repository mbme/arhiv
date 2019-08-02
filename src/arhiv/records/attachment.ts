import { IAttachment } from '~/isodb/types'
import { ArhivReplica } from '../types'

export class Attachment {
  constructor(
    private _replica: ArhivReplica,
    private _attachment: IAttachment,
  ) { }

  get url() {
    return this._replica.getAttachmentUrl(this._attachment._id)
  }

  get id() {
    return this._attachment._id
  }
}
