export type DocumentDataField = { type: 'string' }
| { type: 'markup-string' }
| { type: 'enum', values: readonly string[] }
| { type: 'reference' }

export interface IDocumentDataDescription {
  [name: string]: DocumentDataField
}
