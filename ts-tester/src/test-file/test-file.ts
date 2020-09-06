/* eslint-disable no-console */
import {
  uniq,
  termColors,
} from '@v/utils'
import {
  fileExists,
  readJSON,
  writeJSON,
  removeFile,
} from '@v/utils-node/src/fs'
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

export class TestFile {
  readonly fileName: string

  private constructor(
    private _testContext: TestContext,
    private _updateSnapshots: boolean,
  ) {
    this.fileName = _testContext.testName

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
    if (!await fileExists(this._testContext.snapshotFile)) {
      return {}
    }

    return readJSON(this._testContext.snapshotFile)
  }

  private async _writeSnapshots(snapshots: TestFileSnapshots) {
    if (Object.values(snapshots).length) {
      await writeJSON(this._testContext.snapshotFile, snapshots)
    } else if (await fileExists(this._testContext.snapshotFile)) {
      await removeFile(this._testContext.snapshotFile)
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

      console.log(
        `  ${successfulAsserts.toString().padStart(2, ' ')} ok: ${test.name}`,
        snapshots.length ? `/ ${snapshots.length} snapshots` : '',
        updatedSnapshots ? `  updated ${updatedSnapshots} snapshots` : '',
      )

      return [snapshots, true]
    } catch (e) {
      const {
        successfulAsserts,
      } = getAssertContext()
      if (successfulAsserts) {
        console.log(`  ${successfulAsserts.toString().padStart(2, ' ')} ok: ${test.name}`)
      }
      console.log(termColors.red(` failed: ${test.name}`))
      console.log('')
      console.log(e)
      console.log('')

      return [oldSnapshots, false]
    }
  }

  async run(): Promise<number> {
    const tests = this._getTestsToRun()

    const totalTestsCount = this._testContext.tests.length

    if (tests.length === totalTestsCount) {
      console.log(`${termColors.blue(this.fileName)}`)
    } else {
      const mutedCount = totalTestsCount - tests.length
      console.log(`${termColors.blue(this.fileName)}   ${mutedCount} tests are muted`)
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

    console.log('')

    return failures
  }

  static load(srcPath: string, testFile: string, updateSnapshots: boolean) {
    initializeTestContext(srcPath, testFile)

    // eslint-disable-next-line
    require(testFile) // collect tests from the file into test context

    return new TestFile(getTestContext(), updateSnapshots)
  }
}
