import { Obj } from '@v/utils'

export type DocumentDataField = { type: 'string' }
| { type: 'markup-string' }
| { type: 'enum', values: readonly string[] }
| { type: 'reference' }

export type DocumentDataDescription<P extends Obj> = {
  [name in keyof P]: DocumentDataField
}
