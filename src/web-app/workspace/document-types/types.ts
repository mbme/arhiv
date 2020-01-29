import * as React from 'react'
import { Document } from '~/arhiv/replica'

export interface IDocumentModule {
  readonly type: string,
  matches(document: Document, filter: string): boolean,
  getTitle(document: Document): string,
  renderCard(document: Document): React.ReactNode,
}
