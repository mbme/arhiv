import * as React from 'react'
import { fuzzySearch } from '~/utils'
import { Document } from '~/arhiv/replica'
import { IDocumentModule } from '../types'
import { DocumentCard } from './DocumentCard'

export const DocumentModule: IDocumentModule = {
  type: '',

  getTitle(document: Document): string {
    return `${document.type} ${document.id}`
  },

  matches(document: Document, filter: string): boolean {
    return fuzzySearch(filter, document.id)
  },

  renderCard(document: Document) {
    return (
      <DocumentCard document={document} />
    )
  },
}
