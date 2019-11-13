import { Snapshot } from './types'

export class AssertContext {
  successfulAsserts = 0
  snapshots: Snapshot[] = []
  snapshotPos = 0
  updatedSnapshots = 0

  constructor(
    public readonly oldSnapshots: Snapshot[],
    public readonly updateSnapshots: boolean,
  ) { }
}

let assertContext: AssertContext | undefined

export function getAssertContext(): AssertContext {
  if (!assertContext) {
    throw new Error("Assert context hasn't been initialized yet")
  }

  return assertContext
}

export function initializeAssertContext(oldSnapshots: Snapshot[], updateSnapshot: boolean) {
  assertContext = new AssertContext(oldSnapshots, updateSnapshot)
}
