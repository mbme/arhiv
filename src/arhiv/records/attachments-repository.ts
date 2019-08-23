import { ReactiveValue } from '~/utils/reactive'
import { ArhivReplica } from '../types'
import { Attachment } from './attachment'

export class AttachmentsRepository {
  constructor(protected _replica: ArhivReplica) { }

  createAttachment(file: File): string {
    return this._replica.saveAttachment(file)
  }

  getAttachment(id: string): ReactiveValue<Attachment | undefined> {
    return this._replica.getAttachment(id).map((attachment) => {
      if (attachment) {
        return new Attachment(this._replica, attachment)
      }

      return undefined
    })
  }
}
