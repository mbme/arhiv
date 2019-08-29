// tslint:disable-next-line:match-default-export-name
import assert from 'assert'

export interface IAsserts {
  equal(actual: any, expected: any): void
  deepEqual(actual: any, expected: any): void
  true(actual: any): void
  false(actual: any): void
  matchSnapshot(actual: any): void
  throws(block: () => void, error?: any): void
}

export class Asserts implements IAsserts {
  private _asserts = 0
  private _snapshots: any[] = []
  private _snapshotPos = 0
  private _updatedSnapshots = 0

  constructor(
    private _oldSnapshots: any[],
    private _updateSnapshots: boolean,
  ) { }

  getStats() {
    return {
      asserts: this._asserts,
      snapshots: this._snapshots,
      updatedSnapshots: this._updatedSnapshots,
    }
  }

  equal(actual: any, expected: any) {
    if (actual === expected) {
      this._asserts += 1
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
    assert.deepStrictEqual(actual, expected)
    this._asserts += 1
  }

  true(actual: any) {
    assert.strictEqual(actual, true)
    this._asserts += 1
  }

  false(actual: any) {
    assert.strictEqual(actual, false)
    this._asserts += 1
  }

  matchSnapshot(actual: any) {
    if (this._snapshotPos < this._oldSnapshots.length) {
      try {
        assert.strictEqual(
          JSON.stringify(actual, undefined, 2),
          JSON.stringify(this._oldSnapshots[this._snapshotPos], undefined, 2),
        )
      } catch (e) {
        if (!this._updateSnapshots) throw e
        this._updatedSnapshots += 1
      }
    }

    this._snapshots.push(actual)
    this._snapshotPos += 1
    this._asserts += 1
  }

  throws(block: () => void, error?: any) {
    try {
      block()
      assert.fail('Expected to throw')
    } catch (e) {
      if (error) assert.strictEqual(e, error)
      this._asserts += 1
    }
  }
}
