import { API, IDocument } from '../api'
import { DocumentDataDescription } from '../data-description'

export const NoteDataDescription: DocumentDataDescription<INoteProps> = {
  'name': { type: 'string', title: true },
  'data': { type: 'markup-string' },
}

export interface INoteProps {
  name: string,
  data: string,
}

export type Note = IDocument<'note', INoteProps>

export function createNote(): Promise<Note> {
  return API.create({ documentType: 'note', args: null })
}
