import {
  INote,
  RecordType,
} from '~/isodb-core/types'
import { BaseRecord } from './base-record'

export class Note extends BaseRecord<INote> {
  get name() {
    return this._record.name
  }

  set name(value: string) {
    this._record.name = value
  }

  get data() {
    return this._record.data
  }

  set data(value: string) {
    this._record.data = value
    this.updateRefs(value)
  }

  public static create(id: string): INote {
    return {
      ...BaseRecord.create(id),
      _type: RecordType.Note,
      name: '',
      data: '',
    }
  }

  public static is(x: any): x is INote {
    // tslint:disable-next-line:no-unsafe-any
    return x && x._type === RecordType.Note
  }
}
