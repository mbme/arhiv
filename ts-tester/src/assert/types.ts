import { Dict, Obj } from '@v/utils'

export type Snapshot = Obj | string | number | boolean | null
export type TestFileSnapshots = Dict<Snapshot[]>
