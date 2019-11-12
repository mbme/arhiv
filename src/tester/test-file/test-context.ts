import {
  Procedure,
  AsyncProcedure,
} from '~/utils'
import { Snapshot } from '../types'

type Callback = Procedure | AsyncProcedure

interface ITest {
  name: string
  fn: Callback
  only: boolean
}

export class TestContext {
  beforeCb?: Callback
  afterCb?: Callback
  readonly tests: ITest[] = []

  successfulAsserts = 0
  readonly snapshots: Snapshot[] = []
  snapshotPos = 0
  updatedSnapshots = 0
}

let currentContext: TestContext | undefined

export function initializeTestContext() {
  currentContext = new TestContext()
}

export function getTestContext(): TestContext {
  if (!currentContext) {
    throw new Error("test context isn't initialized yet")
  }

  return currentContext
}
