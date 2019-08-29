// tslint:disable-next-line:match-default-export-name
import assert from 'assert'
import fs from 'fs'
import {
  readJSON,
  writeJSON,
} from '../utils/fs'
import {
  uniq,
  log,
} from '../utils'

type Callback = () => void | Promise<void>
interface IAsserts {
  equal(actual: any, expected: any): void
  deepEqual(actual: any, expected: any): void
  true(actual: any): void
  false(actual: any): void
  matchSnapshot(actual: any): void
  throws(block: () => void, error?: any): void
}
type TestFn = (asserts: IAsserts) => void | Promise<void>
interface ITest {
  name: string
  fn: TestFn
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

export const test = (name: string, fn: TestFn, only = false) => _tests.push({ name, fn, only })
export const before = (cb: Callback) => { _beforeCb = cb }
export const after = (cb: Callback) => { _afterCb = cb }

async function runTest({ name, fn }: ITest, oldSnapshots: any[], updateSnapshots: boolean): Promise<[any[], boolean]> {
  let okAsserts = 0
  let snapshotPos = 0
  const snapshots: any[] = []

  try {
    await Promise.resolve(fn({
      equal(actual: any, expected: any) {
        if (actual === expected) {
          okAsserts += 1
        } else {
          assert.fail(
            `not ok
            expected:
              ${expected}
            actual:
              ${actual}
          `)
        }
      },

      deepEqual(actual: any, expected: any) {
        assert.deepStrictEqual(actual, expected)
        okAsserts += 1
      },

      true(actual: any) {
        assert.strictEqual(actual, true)
        okAsserts += 1
      },

      false(actual: any) {
        assert.strictEqual(actual, false)
        okAsserts += 1
      },

      matchSnapshot(actual: any) {
        if (snapshotPos < oldSnapshots.length) {
          try {
            assert.strictEqual(
              JSON.stringify(actual, undefined, 2),
              JSON.stringify(oldSnapshots[snapshotPos], undefined, 2),
            )
          } catch (e) {
            if (!updateSnapshots) throw e
            log.simple(`  ${name}: updating snapshot`)
          }
        }

        snapshots.push(actual)
        snapshotPos += 1
        okAsserts += 1
      },

      throws(block: () => void, error?: any) {
        try {
          block()
          assert.fail('Expected to throw')
        } catch (e) {
          if (error) assert.strictEqual(e, error)
          okAsserts += 1
        }
      },
    }))

    log.simple(`  ${name}: ${okAsserts} ok`, snapshotPos ? `/ ${snapshotPos} snapshots` : '')

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
