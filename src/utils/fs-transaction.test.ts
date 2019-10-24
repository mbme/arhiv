import fs from 'fs'
import path from 'path'
import {
  after,
  before,
  test,
  asserts,
} from '~/tester'
import { FSTransaction } from './fs-transaction'
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

test('creating file', async () => {
  const t = await FSTransaction.create()
  const file = getRandomFilePath()

  await t.createFile(file, '')
  await t.complete()

  asserts.true(fs.existsSync(file))
})

test('updating file', async () => {
  const t = await FSTransaction.create()
  const file = getRandomFilePath()

  await writeText(file, '1')

  await t.updateFile(file, '2')
  await t.complete()

  asserts.equal(await readText(file), '2')
})

test('deleting file', async () => {
  const t = await FSTransaction.create()
  const file = getRandomFilePath()

  await writeText(file, '1')

  await t.deleteFile(file)
  await t.complete()

  asserts.true(!fs.existsSync(file))
})

test('few operations per transaction', async () => {
  const t = await FSTransaction.create()
  const file1 = getRandomFilePath()
  const file2 = getRandomFilePath()

  await writeText(file2, '1')

  await t.createFile(file1, '')
  await t.updateFile(file2, '2')
  await t.complete()

  asserts.true(fs.existsSync(file1))
  asserts.equal(await readText(file2), '2')
})

test('operations on the same file in transaction', async () => {
  const t = await FSTransaction.create()
  const file1 = getRandomFilePath()

  await t.createFile(file1, '1')
  await t.deleteFile(file1)
  await t.complete()

  asserts.false(fs.existsSync(file1))
})

test('rollback on error', async () => {
  const t = await FSTransaction.create()
  const file1 = getRandomFilePath()
  const file2 = getRandomFilePath()
  const file3 = getRandomFilePath()

  await writeText(file1, '1')
  await writeText(file2, '1')
  await writeText(file3, '1')

  await t.updateFile(file1, '2')
  await t.deleteFile(file2)
  const isErr = await t.createFile(file3, '').catch(e => !!e)

  asserts.true(isErr)
  asserts.equal(await readText(file1), '1')
  asserts.equal(await readText(file2), '1')
})
