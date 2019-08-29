import { createLogger } from '~/utils'
import {
  markupParser,
  selectLinks,
} from '~/markup-parser'
import { stringifyFailure } from '~/parser-combinator'
import {
  Without,
  ReactiveValue,
  Callback,
} from '~/utils'
import { IDocument } from '~/isodb/types'
import {
  ArhivReplica,
  Record,
} from '../types'

const log = createLogger('document')

type LockState = 'pending' | 'acquired' | 'released'
interface ILock {
  $state: ReactiveValue<LockState>
  release: Callback
}

// Active Record
export class Document<T extends Record> {
  constructor(
    protected _replica: ArhivReplica,
    public record: T,
  ) { }

  get id() {
    return this.record._id
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

  $isLocked() {
    return this._replica.locks.$isDocumentLocked(this.id)
  }

  lock(): ILock {
    const $state = new ReactiveValue<LockState>('pending')

    const unsub = this.$isLocked()
      .filter(isLocked => !isLocked)
      .take(1)
      .subscribe({
        next: () => {
          this._replica.locks.addDocumentLock(this.id)
          $state.next('acquired')
        },
      })

    return {
      $state,

      release: () => {
        unsub()

        if ($state.currentValue === 'acquired') {
          this._replica.locks.removeDocumentLock(this.id)
          $state.next('released')
        }
      },
    }
  }
}
