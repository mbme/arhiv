import { IAttachment } from '~/isodb/types'
import { ArhivReplica } from '../types'

export class Attachment {
  constructor(
    private _replica: ArhivReplica,
    public attachment: IAttachment,
  ) { }

  getUrl$() {
    let url = ''

    const url$ = this._replica.getAttachmentData$(this.id).map((blob) => {
      if (url) {
        return url
      }

      if (!blob) {
        return undefined
      }

      url = URL.createObjectURL(blob)

      return url
    })

    url$.subscribe({
      complete: () => {
        if (url) {
          URL.revokeObjectURL(url)
        }
      },
    })

    return url$
  }

  get id() {
    return this.attachment._id
  }
}
