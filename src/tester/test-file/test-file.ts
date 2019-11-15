import path from 'path'
import {
  createLogger,
  uniq,
  termColors,
} from '~/utils'
import {
  fileExists,
  readJSON,
  writeJSON,
  removeFile,
} from '~/utils/fs'
import {
  TestContext,
  initializeTestContext,
  getTestContext,
  ITest,
} from './test-context'
import {
  TestFileSnapshots,
  Snapshot,
} from '../assert/types'
import {
  initializeAssertContext,
  getAssertContext,
} from '../assert/assert-context'

const log = createLogger('tester')

export class TestFile {
  readonly fileName: string
  private _snapshotFile: string

  private constructor(
    private _testContext: TestContext,
    private _updateSnapshots: boolean,
  ) {
    this.fileName = path.relative(_testContext.basePath, _testContext.testFile)
    this._snapshotFile = `${_testContext.testFile}.snap.json`

    // make sure we don't use updateSnapshots option while running only selected tests
    if (_testContext.tests.find(test => test.only) && _updateSnapshots) {
      throw new Error(`${this.fileName} contains "only" tests while updateSnapshots is true`)
    }

    // make sure there are no duplicate tests in the same file
    if (uniq(_testContext.tests, ({ name }) => name).length !== _testContext.tests.length) {
      throw new Error(`${this.fileName} contains tests with similar names`)
    }
  }

  private _getTestsToRun() {
    const onlyTests = this._testContext.tests.filter(test => test.only)

    if (onlyTests.length) {
      return onlyTests
    }

    return this._testContext.tests
  }

  private async _readSnapshots(): Promise<TestFileSnapshots> {
    if (!await fileExists(this._snapshotFile)) {
      return {}
    }

    return readJSON(this._snapshotFile)
  }

  private async _writeSnapshots(snapshots: TestFileSnapshots) {
    if (Object.values(snapshots).length) {
      await writeJSON(this._snapshotFile, snapshots)
    } else if (await fileExists(this._snapshotFile)) {
      await removeFile(this._snapshotFile)
    }
  }

  private async _runTest(test: ITest, oldSnapshots: Snapshot[]): Promise<[Snapshot[], boolean]> {
    try {
      initializeAssertContext(oldSnapshots, this._updateSnapshots)

      await Promise.resolve(test.fn())

      const {
        snapshots,
        updatedSnapshots,
        successfulAsserts,
      } = getAssertContext()

      log.simple(
        `  ${successfulAsserts.toString().padStart(2, ' ')} ok: ${test.name}`,
        snapshots.length ? `/ ${snapshots.length} snapshots` : '',
        updatedSnapshots ? `  updated ${updatedSnapshots} snapshots` : '',
      )

      return [snapshots, true]
    } catch (e) {
      log.simple('')
      log.simple(termColors.red(`  ${test.name}: failed`))
      log.simple('')
      log.simple(e)
      log.simple('')

      return [oldSnapshots, false]
    }
  }

  async run(): Promise<number> {
    const tests = this._getTestsToRun()

    const totalTestsCount = this._testContext.tests.length

    if (tests.length === totalTestsCount) {
      log.simple(`${termColors.blue(this.fileName)}`)
    } else {
      const mutedCount = totalTestsCount - tests.length
      log.simple(`${termColors.blue(this.fileName)}   ${mutedCount} tests are muted`)
    }

    const testTimeout = setTimeout(() => {
      throw new Error('Test is taking too much time, probably due to some race condition.')
    }, 10000)

    if (this._testContext.beforeCb) {
      await Promise.resolve(this._testContext.beforeCb())
    }

    const oldSnapshots = await this._readSnapshots()
    const newSnapshots: TestFileSnapshots = {}

    let failures = 0
    for (const test of tests) {
      const [
        snapshots,
        success,
      ] = await this._runTest(test, oldSnapshots[test.name] || [])

      if (!success) {
        failures += 1
      }

      if (snapshots.length) {
        newSnapshots[test.name] = snapshots
      }
    }

    if (this._testContext.afterCb) {
      await Promise.resolve(this._testContext.afterCb())
    }

    await this._writeSnapshots(newSnapshots)

    clearTimeout(testTimeout)

    log.simple('')

    return failures
  }

  static async load(basePath: string, testFile: string, updateSnapshots: boolean) {
    initializeTestContext(basePath, testFile)

    await import(testFile) // collect tests from the file into test context

    return new TestFile(getTestContext(), updateSnapshots)
  }
}
