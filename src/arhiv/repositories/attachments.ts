import { ArhivReplica } from '../types'
import { Attachment } from '../entities/attachment'

export class AttachmentsRepository {
  constructor(protected _replica: ArhivReplica) { }

  createAttachment(file: File): string {
    return this._replica.saveAttachment(file)
  }

  getAttachment(id: string): Attachment | undefined {
    const attachment = this._replica.getAttachment(id)
    if (!attachment) {
      return undefined
    }

    return new Attachment(this._replica, attachment)
  }
}
