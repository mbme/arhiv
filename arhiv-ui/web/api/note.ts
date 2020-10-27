import { IDocument, API, IDocumentFilter } from './api'

export interface INoteProps {
  name: string,
  data: string,
}

export type Note = IDocument<'note', INoteProps>

export function listNotes(pattern: string): Promise<Note[]> {
  const filter: IDocumentFilter<'note'> = {
    type: 'note',
    matcher: pattern ? {
      selector: '$.name',
      pattern,
    } : undefined,
    skipArchived: true,
  }

  return API.list(filter)
}

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
  return API.create('note')
}
