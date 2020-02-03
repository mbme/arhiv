import * as React from 'react'
import { fuzzySearch } from '~/utils'
import { IDocumentModule } from '../types'
import { NoteCard } from './NoteCard'
import { DocumentNote } from './types'
import { ArhivReplica } from '~/arhiv/replica'

export const NoteModule: IDocumentModule = {
  type: 'note',

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

  create(arhiv: ArhivReplica) {
    return arhiv.documents.create('note', { name: '', data: '' })
  },
}
