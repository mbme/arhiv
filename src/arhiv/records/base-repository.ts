import { LockAgent } from '../agents'
import { ArhivReplica } from '../types'

export abstract class BaseRepository {
  constructor(
    protected _replica: ArhivReplica,
    protected _lockAgent: LockAgent,
  ) { }
}
