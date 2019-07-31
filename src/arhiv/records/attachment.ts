import {
  IAttachment,
} from '~/isodb-core/types'
import { IsodbReplica } from '../replica'

export class Attachment {
  constructor(
    private _replica: IsodbReplica,
    private _attachment: IAttachment,
  ) { }

  get url() {
    return this._replica.getAttachmentUrl(this._attachment._id)
  }

  get id() {
    return this._attachment._id
  }

  save() {
    this._replica.saveAttachment(this._attachment)
  }
}
