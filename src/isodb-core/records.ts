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

function createRecord(replica: IsodbReplica, recordType: RecordType): IRecord {
  const now = nowS()

  return {
    _type: recordType,
    _id: getRandomId(replica),
    _createdTs: now,
    _updatedTs: now,
    _refs: [],
    _attachmentRefs: [],
  }
}


// Active Record
abstract class Record {
  constructor(
    private replica: IsodbReplica,
    protected record: IRecord,
  ) { }

  protected parse(value: string) {

  }

  save() {
    this.replica.saveRecord({
      ...this.record,
      _updatedTs: nowS(),
    })
  }
}


class Note extends Record {
  constructor(replica: IsodbReplica, note?: INote) {
    super(replica, note || createRecord(replica, RecordType.Note))
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

class Track extends Record { }
