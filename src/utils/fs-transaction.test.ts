import fs from 'fs'
import path from 'path'
import {
  after,
  before,
  test,
  asserts,
} from '~/tester'
import { createFsTransaction } from './fs-transaction'
import {
  createTempDir,
  readText,
  rmrfSync,
  writeText,
} from './fs'

let tmpDir: string | undefined
let counter = 0
before(async () => {
  tmpDir = await createTempDir()
})
after(() => {
  rmrfSync(tmpDir!)
})

const getRandomFilePath = () => {
  counter += 1

  return path.join(tmpDir!, counter.toString())
}

test('fails when applying multiple operations to the same file', () => {
  const t = createFsTransaction()
  const file = getRandomFilePath()
  t.addFile(file, '')

  asserts.throws(() => t.updateFile(file, ''))
})

test('adding file', async () => {
  const t = createFsTransaction()
  const file = getRandomFilePath()

  t.addFile(file, '')
  await t.commit()

  asserts.true(fs.existsSync(file))
})

test('updating file', async () => {
  const t = createFsTransaction()
  const file = getRandomFilePath()

  await writeText(file, '1')

  t.updateFile(file, '2')
  await t.commit()

  asserts.equal(await readText(file), '2')
})

test('removing file', async () => {
  const t = createFsTransaction()
  const file = getRandomFilePath()

  await writeText(file, '1')

  t.removeFile(file)
  await t.commit()

  asserts.true(!fs.existsSync(file))
})

test('few operations per transaction', async () => {
  const t = createFsTransaction()
  const file1 = getRandomFilePath()
  const file2 = getRandomFilePath()

  await writeText(file2, '1')

  t.addFile(file1, '')
  t.updateFile(file2, '2')
  await t.commit()

  asserts.true(fs.existsSync(file1))
  asserts.equal(await readText(file2), '2')
})

test('operations on the same file in transaction', async () => {
  const t = createFsTransaction()
  const file1 = getRandomFilePath()

  t.addFile(file1, '1')
  t.removeFile(file1)
  await t.commit()

  asserts.false(fs.existsSync(file1))
})

test('rollback', async () => {
  const t = createFsTransaction()
  const file1 = getRandomFilePath()
  const file2 = getRandomFilePath()
  const file3 = getRandomFilePath()

  await writeText(file1, '1')
  await writeText(file2, '1')
  await writeText(file3, '1')

  t.updateFile(file1, '2')
  t.removeFile(file2)
  t.addFile(file3, '')

  const isErr = await t.commit().catch((e) => !!e)

  asserts.true(isErr)
  asserts.equal(await readText(file1), '1')
  asserts.equal(await readText(file2), '1')
})
