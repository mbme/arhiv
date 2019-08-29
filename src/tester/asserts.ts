// tslint:disable-next-line:match-default-export-name
import assert from 'assert'

interface IState {
  oldSnapshots: any[]
  updateSnapshots: boolean

  successfulAsserts: number
  snapshots: any[]
  snapshotPos: number
  updatedSnapshots: number
}

export class Asserts {
  state?: IState

  init(oldSnapshots: any[], updateSnapshots: boolean) {
    this.state = {
      oldSnapshots,
      updateSnapshots,

      successfulAsserts: 0,
      snapshots: [],
      snapshotPos: 0,
      updatedSnapshots: 0,
    }
  }

  reset() {
    this.state = undefined
  }

  equal(actual: any, expected: any) {
    if (!this.state) {
      throw new Error('asserts not ready')
    }

    if (actual === expected) {
      this.state.successfulAsserts += 1
    } else {
      assert.fail(
        `not ok
        expected:
          ${expected}
        actual:
          ${actual}
      `)
    }
  }

  deepEqual(actual: any, expected: any) {
    if (!this.state) {
      throw new Error('asserts not ready')
    }

    assert.deepStrictEqual(actual, expected)
    this.state.successfulAsserts += 1
  }

  true(actual: any) {
    if (!this.state) {
      throw new Error('asserts not ready')
    }

    assert.strictEqual(actual, true)
    this.state.successfulAsserts += 1
  }

  false(actual: any) {
    if (!this.state) {
      throw new Error('asserts not ready')
    }

    assert.strictEqual(actual, false)
    this.state.successfulAsserts += 1
  }

  matchSnapshot(actual: any) {
    if (!this.state) {
      throw new Error('asserts not ready')
    }

    if (this.state.snapshotPos < this.state.oldSnapshots.length) {
      try {
        assert.strictEqual(
          JSON.stringify(actual, undefined, 2),
          JSON.stringify(this.state.oldSnapshots[this.state.snapshotPos], undefined, 2),
        )
      } catch (e) {
        if (!this.state.updateSnapshots) {
          throw e
        }
        this.state.updatedSnapshots += 1
      }
    }

    this.state.snapshots.push(actual)
    this.state.snapshotPos += 1
    this.state.successfulAsserts += 1
  }

  throws(block: () => void, error?: any) {
    if (!this.state) {
      throw new Error('asserts not ready')
    }

    try {
      block()
      assert.fail('Expected to throw')
    } catch (e) {
      if (error) {
        assert.strictEqual(e, error)
      }
      this.state.successfulAsserts += 1
    }
  }
}
