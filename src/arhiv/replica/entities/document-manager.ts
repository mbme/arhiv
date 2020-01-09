import { createLogger } from '~/logger'
import {
  selectLinks,
  parseMarkup,
} from '~/markup-parser'
import {
  IDocument,
} from '~/arhiv/types'
import {
  ReplicaDB,
} from '../db'
import { LockManager } from '../managers'

const log = createLogger('document')

// Active Record
export class DocumentManager<T extends string = string, P extends object = {}> {
  constructor(
    private _db: ReplicaDB,
    private _locks: LockManager,
    private _document: IDocument<T, P>,
    private _isNew: boolean = false,
  ) { }

  get type(): T {
    return this._document.type
  }

  get id(): string {
    return this._document.id
  }

  get createdAt(): Date {
    return new Date(this._document.createdAt)
  }

  get updatedAt(): Date {
    return new Date(this._document.updatedAt)
  }

  get props(): Readonly<P> {
    return this._document.props
  }

  private async _extractRefs(value: string): Promise<string[]> {
    const attachmentRefs = new Set<string>()

    const markup = parseMarkup(value)

    for (const link of selectLinks(markup)) {
      const id = link.link

      if (await this._db.getAttachment(id)) {
        attachmentRefs.add(id)
      } else {
        log.warn(`document ${this.id} references unknown entity ${id}`)
      }
    }

    return Array.from(attachmentRefs)
  }

  async patch(patch: Partial<P>, _refProps: Array<keyof P>) {
    const patchedProps = {
      ...this._document.props,
      ...patch,
    }
    const refSources = _refProps.map(prop => patchedProps[prop])

    // FIXME extract doc refs
    // FIXME do not join sources
    const attachmentRefs = refSources.length
      ? await this._extractRefs(refSources.join(' '))
      : this._document.attachmentRefs

    await this._db.saveDocument({
      ...this._document,
      ...patch,
      _attachmentRefs: attachmentRefs,
    })
    this._isNew = false
  }

  async delete() {
    await this._db.saveDocument({
      ...this._document,
      deleted: true,
    })
  }

  isNew() {
    return this._isNew
  }

  isLocked$() {
    return this._locks.isDocumentLocked$(this.id)
  }

  acquireLock$() {
    return this._locks.acquireDocumentLock$(this.id)
  }
}
