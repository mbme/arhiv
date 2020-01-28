import { Document } from '~/arhiv'

interface IDocumentModule {
  readonly type: string,
}

export const type = 'note'

export function getTitle(document: Document): string {
}
