import { Obj } from '@v/utils'
import { IDocument } from './api'

export type DocumentDataField = { type: 'string', title?: true }
| { type: 'markup-string' }
| { type: 'enum', values: readonly string[] }
| { type: 'reference' }

export type DocumentDataDescription<P extends Obj> = {
  [name in keyof P]: DocumentDataField
}

export function pickTitle<P extends Obj>(
  document: IDocument<string, P>,
  dataDescription: DocumentDataDescription<P>,
): string {
  const titleEntry = Object.entries(dataDescription).find(([, field]) => field.type === 'string' && field.title)

  if (!titleEntry) {
    return `${document.type} #{document.id}`
  }

  const [name] = titleEntry

  return document.data[name] as string
}
