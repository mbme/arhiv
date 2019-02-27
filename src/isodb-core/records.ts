import {
  nowS,
} from '~/utils'
import {
  IRecord,
  INote,
  RecordType,
} from './types'
import { randomId } from '~/randomizer'
import IsodbReplica from './replica';

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

function getRandomId(replica: IsodbReplica) {
  let id: string

  do {
    id = randomId(ID_ALPHABET, ID_LENGTH)
  } while (replica.getRecord(id)) // make sure generated id is free

  return id
}

function createRecord(replica: IsodbReplica) {
  const now = nowS()

  return {
    _id: getRandomId(replica),
    _createdTs: now,
    _updatedTs: now,
    _refs: [],
    _attachmentRefs: [],
  }
}


// Active Record
abstract class Record<T extends IRecord> {
  protected record: T
  constructor(
    protected replica: IsodbReplica,
    record?: T,
  ) {
    this.record = record || this.create()
  }

  protected parse(_value: string) {
    return {
      refs: [],
      attachmentRefs: [],
    }
  }

  protected abstract create(): T

  setDeleted(deleted = true) {
    this.record._deleted = deleted
  }

  save() {
    this.replica.saveRecord({
      ...this.record,
      _updatedTs: nowS(),
    })
  }
}


class Note extends Record<INote> {
  create(): INote {
    return {
      ...createRecord(this.replica),
      _type: RecordType.Note,
      name: '',
      data: '',
    }
  }

  get name() {
    return this.record.name
  }

  set name(value: string) {
    this.record.name = value
  }

  get data() {
    return this.record.data
  }

  set data(value: string) {
    const {
      refs,
      attachmentRefs,
    } = this.parse(value)

    this.record.data = value
    this.record._refs = refs
    this.record._attachmentRefs = attachmentRefs
  }
}

class Track extends Record {
}
