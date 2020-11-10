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

export function createNote(): Promise<Note> {
  return API.create({ documentType: 'note', args: null })
}
