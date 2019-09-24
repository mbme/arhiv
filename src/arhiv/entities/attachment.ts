import { IAttachment } from '~/isodb/types'
import { ReactiveValue } from '~/utils/reactive'
import { ArhivReplica } from '../types'

export class Attachment {
  constructor(
    private _replica: ArhivReplica,
    public attachment: IAttachment,
  ) { }

  getUrl$() {
    return new ReactiveValue<string | undefined>(undefined, (observer) => {
      let url = ''
      const unsub = this._replica.getAttachmentData$(this.id)
        .filter(blob => !!blob)
        .take(1)
        .subscribe({
          next(blob) {
            url = URL.createObjectURL(blob)
            observer.next(url)
          },
        })

      return () => {
        unsub()
        if (url) {
          URL.revokeObjectURL(url)
        }
      }
    })
  }

  get id() {
    return this.attachment._id
  }
}
