import {
  assert,
} from '~/tester'
import {
  Result,
  isSuccess,
  isFailure,
} from './parser'

export const assertSuccess = <T>(r: Result<T>, cb?: (result: T) => void) => {
  assert.true(isSuccess(r))

  if (cb && isSuccess(r)) {
    cb(r.result)
  }
}

export const assertFailure = <T>(r: Result<T>) => {
  assert.true(isFailure(r))
}
