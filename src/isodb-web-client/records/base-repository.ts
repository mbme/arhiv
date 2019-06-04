import { generateRandomId } from '~/isodb-core/utils'
import { IsodbReplica } from '../replica'
import { LockAgent } from '../agents'

export abstract class BaseRepository {
  constructor(
    protected _replica: IsodbReplica,
    protected _lockAgent: LockAgent,
  ) { }

  protected getRandomId() {
    let id: string

    do {
      id = generateRandomId()
    } while (this._replica.getRecord(id)) // make sure generated id is free

    return id
  }

}
