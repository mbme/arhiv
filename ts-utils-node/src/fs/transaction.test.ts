import fs from 'fs'
import path from 'path'
import {
  after,
  before,
  test,
  assertEqual,
  assertTrue,
  assertFalse,
} from '@v/tester'
import { FSTransaction } from './transaction'
import {
  createTempDir,
  readText,
  rmrfSync,
  writeText,
  isDirectory,
} from './utils'

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
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await t.createFile(file, '')
  await t.complete()

  assertTrue(fs.existsSync(file))
})

test('revert creating file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await t.createFile(file, '')
  await t.revert()

  assertFalse(fs.existsSync(file))
})

test('updating file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await writeText(file, '1')

  await t.updateFile(file, '2')
  await t.complete()

  assertEqual(await readText(file), '2')
})

test('revert updating file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await writeText(file, '1')

  await t.updateFile(file, '2')
  await t.revert()

  assertEqual(await readText(file), '1')
})

test('moving file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()
  const newFile = getRandomFilePath()

  await writeText(file, '1')

  await t.moveFile(file, newFile)
  await t.complete()

  assertFalse(fs.existsSync(file))
  assertEqual(await readText(newFile), '1')
})

test('revert moving file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()
  const newFile = getRandomFilePath()

  await writeText(file, '1')

  await t.moveFile(file, newFile)
  await t.revert()

  assertFalse(fs.existsSync(newFile))
  assertEqual(await readText(file), '1')
})

test('deleting file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await writeText(file, '1')

  await t.deleteFile(file)
  await t.complete()

  assertFalse(fs.existsSync(file))
})

test('revert deleting file', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await writeText(file, '1')

  await t.deleteFile(file)
  await t.revert()

  assertEqual(await readText(file), '1')
})

test('creating directory', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await t.createDir(file)
  await t.complete()

  assertTrue(fs.existsSync(file))
  assertTrue(await isDirectory(file))
})

test('revert creating directory', async () => {
  const t = new FSTransaction()
  const file = getRandomFilePath()

  await t.createDir(file)
  await t.revert()

  assertFalse(fs.existsSync(file))
})

test('few operations per transaction', async () => {
  const t = new FSTransaction()
  const file1 = getRandomFilePath()
  const file2 = getRandomFilePath()

  await writeText(file2, '1')

  await t.createFile(file1, '')
  await t.updateFile(file2, '2')
  await t.complete()

  assertTrue(fs.existsSync(file1))
  assertEqual(await readText(file2), '2')
})

test('operations on the same file in transaction', async () => {
  const t = new FSTransaction()
  const file1 = getRandomFilePath()

  await t.createFile(file1, '1')
  await t.deleteFile(file1)
  await t.complete()

  assertFalse(fs.existsSync(file1))
})
