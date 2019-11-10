// tslint:disable-next-line:match-default-export-name
import assert from 'assert'
import { Constructor } from '~/utils'

interface IState {
  oldSnapshots: any[]
  updateSnapshots: boolean

  successfulAsserts: number
  snapshots: any[]
  snapshotPos: number
  updatedSnapshots: number
}

export class Asserts {
  _state?: IState

  init(oldSnapshots: any[], updateSnapshots: boolean) {
    this._state = {
      oldSnapshots,
      updateSnapshots,

      successfulAsserts: 0,
      snapshots: [],
      snapshotPos: 0,
      updatedSnapshots: 0,
    }
  }

  reset() {
    this._state = undefined
  }

  equal(actual: any, expected: any) {
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    if (actual === expected) {
      this._state.successfulAsserts += 1
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
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    assert.deepStrictEqual(actual, expected)
    this._state.successfulAsserts += 1
  }

  true(actual: any) {
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    assert.strictEqual(actual, true)
    this._state.successfulAsserts += 1
  }

  false(actual: any) {
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    assert.strictEqual(actual, false)
    this._state.successfulAsserts += 1
  }

  matchSnapshot(actual: any) {
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    if (this._state.snapshotPos < this._state.oldSnapshots.length) {
      try {
        assert.strictEqual(
          JSON.stringify(actual, undefined, 2),
          JSON.stringify(this._state.oldSnapshots[this._state.snapshotPos], undefined, 2),
        )
      } catch (e) {
        if (!this._state.updateSnapshots) {
          throw e
        }
        this._state.updatedSnapshots += 1
      }
    }

    this._state.snapshots.push(actual)
    this._state.snapshotPos += 1
    this._state.successfulAsserts += 1
  }

  throws(block: () => void, error?: any) {
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    try {
      block()
      assert.fail('Expected to throw')
    } catch (e) {
      if (error) {
        assert.strictEqual(e, error)
      }
      this._state.successfulAsserts += 1
    }
  }

  // FIXME this doesn't work due to https://github.com/microsoft/TypeScript/issues/35004
  instanceOf<T>(value: unknown, classConstructor: Constructor<T>): asserts value is T {
    if (!this._state) {
      throw new Error('asserts not ready')
    }

    if (value instanceof classConstructor) {
      this._state.successfulAsserts += 1
    } else {
      assert.fail(`Expected to be instance of ${classConstructor}`)
    }
  }
}
