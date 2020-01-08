import {
  isString,
  fuzzySearch,
} from '~/utils'
import { IDocument } from '~/arhiv/schema'
import { createDocument } from '~/arhiv/utils'
import { DocumentManager } from './document-manager'
import { ReplicaDB } from '../db'
import { LockManager } from '../managers'

export interface INoteProps {
  name: string
  data: string
}

const initialProps = {
  name: '',
  data: '',
}

export class NoteManager extends DocumentManager<'note', INoteProps> {
  constructor(
    db: ReplicaDB,
    locks: LockManager,
    documentOrId: string | IDocument<'note', INoteProps>,
  ) {
    if (isString(documentOrId)) {
      super(db, locks, createDocument(documentOrId, 'note', initialProps), true)
    } else {
      super(db, locks, documentOrId, false)
    }
  }

  protected _isMarkupField(field: string) {
    return field === 'data'
  }

  matches(query: string) {
    return fuzzySearch(query, this.document.props.name)
  }

  getTitle() {
    return this.document.props.name
  }
}
