import { IDict } from '~/utils'

export type Snapshot = object | string | number | boolean | null
export type TestFileSnapshots = IDict<Snapshot[]>
