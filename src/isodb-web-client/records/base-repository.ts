import { IsodbReplica } from '../replica'
import { LockAgent } from '../agents'

export abstract class BaseRepository {
  constructor(
    protected _replica: IsodbReplica,
    protected _lockAgent: LockAgent,
  ) { }
}
