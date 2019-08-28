import { createLogger } from '~/logger'
import {
  markupParser,
  selectLinks,
} from '~/markup-parser'
import { stringifyFailure } from '~/parser-combinator'
import { ReactiveValue } from '~/utils/reactive'
import { Without } from '~/utils'
import { IDocument } from '~/isodb/types'
import {
  ArhivReplica,
  Record,
} from '../types'

const log = createLogger('document')

// Active Record
export class Document<T extends Record> {
  $locked: ReactiveValue<boolean>

  constructor(
    protected _replica: ArhivReplica,
    public record: T,
  ) {
    this.$locked = this._replica.locks.$isDocumentLocked(record._id)
  }

  private _extractRefs(value: string) {
    const attachmentRefs: string[] = []

    const result = markupParser.parseAll(value)
    if (!result.success) {
      throw new Error(`Failed to parse markup: ${stringifyFailure(result)}`)
    }

    for (const link of selectLinks(result.result)) {
      const id = link.value[0]

      if (this._replica.getAttachment(id).currentValue) {
        attachmentRefs.push(id)
      } else {
        log.warn(`document ${this.id} references unknown entity ${id}`)
      }
    }

    return attachmentRefs
  }

  patch(patch: Partial<Without<T, keyof IDocument>>, refSource?: string) { // FIXME fix type
    const attachmentRefs = refSource === undefined
      ? this.record._attachmentRefs
      : this._extractRefs(refSource)

    this._replica.saveDocument({
      ...this.record,
      ...patch,
      _attachmentRefs: attachmentRefs,
    })
  }

  delete() {
    this._replica.saveDocument({ // FIXME cleanup fields
      ...this.record,
      _deleted: true,
    })
  }

  isNew() {
    return !this._replica.getDocument(this.id)
  }

  $lock() {
    return new ReactiveValue(false, (next, error, complete) => {
      let destroy: () => void

      const unsubscribe = this._replica.locks.$isDocumentLocked(this.id).subscribe(
        (isLocked) => {
          if (isLocked) {
            next(false)
          } else {
            unsubscribe()
            this._replica.locks.addDocumentLock(this.id)
            destroy = () => this._replica.locks.removeDocumentLock(this.id)
            next(true)
          }
        },
        error,
        complete,
      )

      return () => (destroy || unsubscribe)()
    })
  }

  get id() {
    return this.record._id
  }
}
