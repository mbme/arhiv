import path from 'path'
import {
  Procedure,
  AsyncProcedure,
  trimSuffix,
} from '@v/utils'
import { TestFileSnapshots } from '../assert/types'

type Callback = Procedure | AsyncProcedure

export interface ITest {
  name: string
  fn: Callback
  only: boolean
}

export class TestContext {
  beforeCb?: Callback
  afterCb?: Callback
  readonly tests: ITest[] = []

  successfulAsserts = 0
  readonly snapshots: TestFileSnapshots[] = []
  snapshotPos = 0
  updatedSnapshots = 0

  constructor(
    public readonly testName: string,
    public readonly testFile: string,
    public readonly snapshotFile: string,
  ) { }
}

let currentContext: TestContext | undefined

export function initializeTestContext(srcPath: string, testFile: string) {
  const relPath = trimSuffix(path.relative(srcPath, testFile), '.ts')
  const snapshotFile = `${path.join(srcPath, relPath)}.snap.json`

  currentContext = new TestContext(relPath, testFile, snapshotFile)
}

export function getTestContext(): TestContext {
  if (!currentContext) {
    throw new Error("test context isn't initialized yet")
  }

  return currentContext
}

export function test(name: string, fn: Callback, only = false) {
  getTestContext().tests.push({ name, fn, only })
}

export function before(cb: Callback) {
  getTestContext().beforeCb = cb
}

export function after(cb: Callback) {
  getTestContext().afterCb = cb
}
