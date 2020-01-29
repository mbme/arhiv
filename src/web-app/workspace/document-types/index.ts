import { Document } from '~/arhiv/replica'
import { IDocumentModule } from './types'
import { NoteModule } from './note'
import { DocumentModule } from './document'

const Modules: IDocumentModule[] = [
  NoteModule,
]

function findModule(type: string): IDocumentModule {
  for (const Module of Modules) {
    if (Module.type === type) {
      return Module
    }
  }

  return DocumentModule
}

export function matches(filter: string) {
  return (document: Document) => findModule(document.type).matches(document, filter)
}

export function getTitle(document: Document) {
  return findModule(document.type).getTitle(document)
}

export function renderCard(document: Document) {
  return findModule(document.type).renderCard(document)
}
