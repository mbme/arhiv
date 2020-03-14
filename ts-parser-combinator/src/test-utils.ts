import {
  assertTrue,
} from '@v/tester'
import {
  Result,
  isSuccess,
  isFailure,
  Success,
  Failure,
} from './parser'

export function assertSuccess<T>(r: Result<T>): asserts r is Success<T> {
  assertTrue(isSuccess(r), 'Expected value to be Success')
}

export function assertFailure(r: Result<any>): asserts r is Failure {
  assertTrue(isFailure(r), 'Expected value to be Failure')
}
