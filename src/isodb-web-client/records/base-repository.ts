import { generateRecordId } from '~/isodb-core/utils'
import { IsodbReplica } from '../replica'

export abstract class BaseRepository {
  constructor(protected _replica: IsodbReplica) { }

  protected getRandomId() {
    let id: string

    do {
      id = generateRecordId()
    } while (this._replica.getRecord(id)) // make sure generated id is free

    return id
  }

}
