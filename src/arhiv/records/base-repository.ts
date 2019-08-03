import { ArhivReplica } from '../types'

export abstract class BaseRepository {
  constructor(
    protected _replica: ArhivReplica,
  ) { }
}
