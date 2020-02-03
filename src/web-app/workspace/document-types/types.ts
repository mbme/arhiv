import * as React from 'react'
import { Document, ArhivReplica } from '~/arhiv/replica'

export interface IDocumentModule {
  readonly type: string,
  matches(document: Document, filter: string): boolean,
  getTitle(document: Document): string,
  renderCard(document: Document): React.ReactNode,
  create(arhiv: ArhivReplica): Promise<Document>,
}
