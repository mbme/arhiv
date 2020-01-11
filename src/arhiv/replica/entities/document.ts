import { createLogger } from '~/logger'
import {
  Dict,
  fuzzySearch,
} from '~/utils'
import { Observable } from '~/reactive'
import { IDocument } from '~/arhiv/types'
import {
  parseMarkup,
  selectLinks,
} from '~/arhiv/markup-parser'
import { ReplicaDB } from '../db'

const log = createLogger('document')

export abstract class Document<P extends object = Dict<any>> {
  constructor(
    private _db: ReplicaDB,
    protected _document: IDocument<string, P>,
  ) {
  }

  get type(): string {
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

  isNew$(): Observable<boolean> {
    return this._db.getDocument$(this.id, false).map(document => document === undefined)
  }

  protected async _updateRefs(...markupStrings: string[]): Promise<void> {
    const attachmentRefs = new Set<string>()
    const documentRefs = new Set<string>()

    for (const value of markupStrings) {
      const markup = parseMarkup(value)

      for (const link of selectLinks(markup)) {
        const id = link.link

        if (await this._db.getDocument(id)) {
          documentRefs.add(id)
        } else if (await this._db.getAttachment(id)) {
          attachmentRefs.add(id)
        } else {
          log.warn(`document ${this.id} references unknown entity ${id}`)
        }
      }
    }

    this._document = {
      ...this._document,
      refs: Array.from(documentRefs),
      attachmentRefs: Array.from(attachmentRefs),
    }
  }

  delete() {
    this._document = {
      ...this._document,
      deleted: true,
    }
  }

  async save() {
    await this._db.saveDocument(this._document)
  }

  matches(filter: string) {
    return fuzzySearch(filter, this.getTitle())
  }

  abstract getTitle(): string
}
