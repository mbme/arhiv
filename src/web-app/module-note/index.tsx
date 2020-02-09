import * as React from 'react'
import { fuzzySearch } from '~/utils'
import { NoteCard } from './NoteCard'
import { DocumentNote } from './types'
import { IDocumentModule } from '../workspace/modules'

export const NoteModule: IDocumentModule = {
  type: 'note',

  initialProps: {
    name: '',
    data: '',
  },

  getTitle(document: DocumentNote): string {
    return document.props.name
  },

  matches(document: DocumentNote, filter: string): boolean {
    return fuzzySearch(filter, document.props.name)
  },

  renderCard(document: DocumentNote) {
    return (
      <NoteCard document={document} />
    )
  },
}
