import { test } from '~/tester'
import { nowS } from '~/utils'
import { RecordType, INote } from '~/isodb-core/types'
import PrimaryDB from './primary'
import PrimaryInMemStorage from './primary-in-mem-storage'

function createNote(id: string, rev?: number): INote {
  return {
    _id: id,
    _type: RecordType.Note,
    _rev: rev,
    _refs: [],
    _attachmentRefs: [],
    _createdTs: nowS(),
    _updatedTs: nowS(),
    name: 'test',
    data: 'test',
  }
}

function initDB(size: number) {
  const storage = new PrimaryInMemStorage()
  storage._rev = size - 1
  for (let i = 0; i < size; i += 1) {
    storage._records.push(createNote(i.toString(), i))
  }

  return {
    storage,
    db: new PrimaryDB(storage),
  }
}

test('getRecords', (assert) => {
  const { db } = initDB(2)
  const records = db.getRecords()

  assert.equal(records.length, 2)
  assert.equal(records[0]._id, '0')
})

test('getRecord', (assert) => {
  const { db } = initDB(2)
  assert.false(!!db.getRecord('999'))
  assert.true(!!db.getRecord('1'))
})

test('applyChangeset', (assert) => {
  const { db } = initDB(2)

  // wrong revision
  assert.throws(() => {
    db.applyChangeset({ baseRev: 2, records: [], attachments: [] }, {})
  })
  assert.throws(() => {
    db.applyChangeset({ baseRev: 0, records: [], attachments: [] }, {})
  })
  assert.throws(() => {
    db.applyChangeset({ baseRev: 1, records: [], attachments: [] }, {})
  })

  // update
  db.applyChangeset({ baseRev: 1, records: [createNote('0')], attachments: [] }, {})

  // add
  db.applyChangeset({ baseRev: 2, records: [createNote('2')], attachments: [] }, {})

  assert.equal(db.getRev(), 3)
  assert.equal(db.getRecords().length, 3)

  // add attachment
  db.applyChangeset({ baseRev: 3, records: [], attachments: [{ _id: '3' }] }, { 3: '/path' })
  assert.true(!!db.getAttachment('3'))

  // update attachment data should fail
  assert.throws(() => {
    db.applyChangeset({ baseRev: 4, records: [], attachments: [{ _id: '3' }] }, { 3: '/path1' })
  })
})
