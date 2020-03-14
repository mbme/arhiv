import { Dict } from '@v/utils'

export type Snapshot = object | string | number | boolean | null
export type TestFileSnapshots = Dict<Snapshot[]>
