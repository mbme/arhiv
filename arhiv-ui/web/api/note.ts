import { API } from './api'
import { IDocument } from './types'
import { DocumentDataDescription } from '../data-description'

export const NoteDataDescription: DocumentDataDescription<INoteProps> = {
  'name': { type: 'string' },
  'data': { type: 'markup-string' },
}

export interface INoteProps {
  name: string,
  data: string,
}

export type Note = IDocument<'note', INoteProps>

export async function getNote(id: string): Promise<Note | null> {
  const document = await API.get(id)

  if (!document) {
    return null
  }

  if (document.type !== 'note') {
    throw new Error(`Document ${id} isn't a note`)
  }

  return document as Note
}

export function createNote(): Promise<Note> {
  return API.create({ documentType: 'note' })
}
