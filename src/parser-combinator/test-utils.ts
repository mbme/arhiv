import {
  asserts,
} from '~/tester'
import {
  Result,
  isSuccess,
  isFailure,
} from './parser'

export const assertSuccess = <T>(r: Result<T>, cb?: (result: T) => void) => {
  asserts.true(isSuccess(r))

  if (cb && isSuccess(r)) {
    cb(r.result)
  }
}

export const assertFailure = <T>(r: Result<T>) => {
  asserts.true(isFailure(r))
}
