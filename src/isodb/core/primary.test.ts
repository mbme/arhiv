import { test } from '~/tester'
import PrimaryDB from './primary'
import PrimaryInMemStorage from './primary-in-mem-storage'

function initDB(size: number) {
  const storage = new PrimaryInMemStorage()
  storage._rev = size - 1
  for (let i = 0; i < size; i += 1) {
    storage._records.push({
      _id: `${i}`,
      _type: 'note',
      _rev: i,
      _refs: [],
      _attachmentRefs: [],
      name: 'test',
      data: 'test',
    })
  }

  return {
    storage,
    db: new PrimaryDB(storage),
  }
}

test('getAll', (assert) => {
  const { db } = initDB(2)
  const records = db.getPatch(1).records

  assert.equal(records.length, 2)
  assert.equal(records[0], '0')
})

test('getRecord', (assert) => {
  const { db } = initDB(2)
  assert.false(!!db.getRecord('999'))
  assert.true(!!db.getRecord('1'))
})

test('applyChanges', (assert) => {
  const { db } = initDB(2)

  // wrong revision
  assert.false(db.applyChanges(2, []))
  assert.false(db.applyChanges(0, []))
  assert.true(db.applyChanges(1, []))

  // update
  assert.true(db.applyChanges(1, [{ _id: '0', _refs: [] }]))

  // add
  assert.true(db.applyChanges(2, [{ _id: '2', _refs: [] }]))

  assert.equal(db.getRev(), 3)
  assert.equal(db.getAll().length, 3)

  // add attachment
  assert.true(db.applyChanges(3, [{ _id: '3', _attachment: true }], { 3: '/path' }))
  assert.equal(db.getAttachment('3'), '/path')

  // update attachment data should fail
  assert.throws(() => {
    db.applyChanges(4, [{ _id: '3', _attachment: true }], { 3: '/path1' })
  })
})

test('compact when nothing to compact', (assert) => {
  const { db } = initDB(2)
  db.compact()
  assert.equal(db.getAll().length, 2)
  assert.equal(db.getRev(), 1)
})

test('compact deleted but still referenced record', (assert) => {
  const { db, storage } = initDB(2)
  storage._records[0]._refs = ['1']
  storage._records[1]._deleted = true
  db.compact()
  assert.equal(db.getAll().length, 2)
  assert.equal(db.getRev(), 1)
})

test('compact deleted record', (assert) => {
  const { db, storage } = initDB(2)
  storage._records[1]._deleted = true
  db.compact()
  assert.equal(db.getAll().length, 1)
  assert.equal(db.getRev(), 2)
})

test('compact deleted record referenced only by another deleted record', (assert) => {
  const { db, storage } = initDB(2)
  storage._records[0]._deleted = true
  storage._records[0]._refs = ['1']
  storage._records[1]._deleted = true
  db.compact()
  assert.equal(db.getAll().length, 0)
  assert.equal(db.getRev(), 2)
})

test('compact attachments not referenced by records', (assert) => {
  const { db, storage } = initDB(2)
  db.applyChanges(1, [
    { _id: '3', _attachment: true },
    { _id: '4', _attachment: true },
  ], { 3: '/path', 4: '/path' })
  storage._records[0]._refs = ['3']
  db.compact()
  assert.equal(db.getAll().length, 3)
  assert.equal(db.getRev(), 3)
})
