import { createDocument } from '~/arhiv/utils'
import { Document } from './document'
import { ReplicaDB } from '../db'

export interface INoteProps {
  name: string
  data: string
}

export class DocumentNote extends Document<INoteProps> {
  static type = 'note'

  static async create(db: ReplicaDB) {
    const id = await db.getRandomId()

    const document = createDocument(id, DocumentNote.type, {
      name: '',
      data: '',
    })

    return new DocumentNote(db, document)
  }

  async patch(patch: Partial<INoteProps>) {
    this._document = {
      ...this._document,
      props: {
        ...this._document.props,
        ...patch,
      },
    }

    await this._updateRefs(this._document.props.data)
  }

  getTitle() {
    return this.props.name
  }

  get name() {
    return this.props.name
  }

  get data() {
    return this.props.data
  }
}
