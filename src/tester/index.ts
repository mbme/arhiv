import fs from 'fs'
import {
  readJSON,
  writeJSON,
} from '~/utils/fs'
import {
  uniq,
  AsyncProcedure,
  Procedure,
  createLogger,
} from '~/utils'
import { Asserts } from './asserts'

const log = createLogger('tester')

type Callback = Procedure | AsyncProcedure

interface ITest {
  name: string
  fn: Callback
  only: boolean
}

let _beforeCb: Callback | undefined
let _tests: ITest[] = []
let _afterCb: Callback | undefined

export function initTestPlan() {
  _beforeCb = undefined
  _tests = []
  _afterCb = undefined
}

export function getTestPlan() {
  return {
    tests: _tests,
    before: _beforeCb,
    after: _afterCb,
  }
}

export const test = (name: string, fn: Callback, only = false) => _tests.push({ name, fn, only })
export const before = (cb: Callback) => { _beforeCb = cb }
export const after = (cb: Callback) => { _afterCb = cb }

export const assert = new Asserts()
async function runTest({ name, fn }: ITest, oldSnapshots: any[], updateSnapshots: boolean): Promise<[any[], boolean]> {
  try {
    assert.init(oldSnapshots, updateSnapshots)

    await Promise.resolve(fn())

    const {
      snapshots,
      updatedSnapshots,
      successfulAsserts,
    } = assert._state!

    log.simple(
      `  ${successfulAsserts.toString().padStart(2, ' ')} ok: ${name}`,
      snapshots.length ? `/ ${snapshots.length} snapshots` : '',
      updatedSnapshots ? `  updated ${updatedSnapshots} snapshots` : '',
    )

    assert.reset()

    return [snapshots, true]
  } catch (e) {
    log.simple(`\n  ${name}: failed\n`, e, '\n')

    return [oldSnapshots, false]
  }
}

export async function runTests(file: string, tests: ITest[], updateSnapshots: boolean) {
  if (uniq(tests, ({ name }) => name).length !== tests.length) {
    throw new Error(`${file} contains tests with similar names`)
  }

  const snapshotsFile = `${file}.snap.json`
  const snapshotsFileExists = fs.existsSync(snapshotsFile)
  const oldSnapshots: { [name: string]: any[] } = snapshotsFileExists ? await readJSON(snapshotsFile) : {}

  const newSnapshots: { [key: string]: any } = {}
  let failures = 0

  for (const t of tests) {
    const [snapshots, success] = await runTest(t, oldSnapshots[t.name] || [], updateSnapshots)

    if (!success) failures += 1

    if (snapshots.length) {
      newSnapshots[t.name] = snapshots
    }
  }

  if (Object.values(newSnapshots).length) {
    await writeJSON(snapshotsFile, newSnapshots)
  } else if (snapshotsFileExists) {
    await fs.promises.unlink(snapshotsFile)
  }

  return failures
}
