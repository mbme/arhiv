import { strict as assert } from 'assert' // https://nodejs.org/api/assert.html#assert_strict_mode
import { Snapshot } from './types'
import { getAssertContext } from './assert-context'
import {
  prettyPrintJSON,
  Constructor,
} from '@v/utils'

export function assertEqual(actual: any, expected: any) {
  const context = getAssertContext()

  if (actual === expected) {
    context.successfulAsserts += 1
  } else {
    assert.fail(
      `not ok
      expected:
      ${expected}
      actual:
      ${actual}
      `,
    )
  }
}

export function assertDeepEqual(actual: any, expected: any) {
  const context = getAssertContext()

  assert.deepStrictEqual(actual, expected)
  context.successfulAsserts += 1
}

export function assertTrue(actual: any, msg?: string) {
  const context = getAssertContext()

  assert.strictEqual(actual, true, msg)
  context.successfulAsserts += 1
}

export function assertFalse(actual: any) {
  const context = getAssertContext()

  assert.strictEqual(actual, false)
  context.successfulAsserts += 1
}

export function assertMatchSnapshot(actual: Snapshot) {
  const context = getAssertContext()

  if (context.snapshotPos < context.oldSnapshots.length) {
    try {
      assert.strictEqual(
        prettyPrintJSON(actual),
        prettyPrintJSON(context.oldSnapshots[context.snapshotPos]),
      )
    } catch (e) {
      if (!context.updateSnapshots) {
        throw e
      }
      context.updatedSnapshots += 1
    }
  }

  context.snapshots.push(actual)
  context.snapshotPos += 1
  context.successfulAsserts += 1
}

export function assertThrows(block: () => void, ErrorClass?: any) {
  const context = getAssertContext()

  try {
    block()
    assert.fail('Expected to throw')
  } catch (e) {
    if (ErrorClass) {
      assertTrue(e instanceof ErrorClass, `Expected error to be instance of ${ErrorClass}`)
    }

    context.successfulAsserts += 1
  }
}

export function assertInstanceOf<T>(value: unknown, classType: Constructor<T>): asserts value is T {
  assertTrue(value instanceof classType, `Expected value to be instance of ${classType}`)
}
