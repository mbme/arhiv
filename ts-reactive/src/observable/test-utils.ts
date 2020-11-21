import {
  assertDeepEqual,
} from '@v/tester'
import { Observable } from './observable'

export const complete = Symbol('complete')
export const error = Symbol('error')

type ResultArray<T> = Array<T | typeof complete | typeof error>

export async function observableToArray<T>(o$: Observable<T>): Promise<ResultArray<T>> {
  const result: ResultArray<T> = []

  await new Promise<void>((resolve) => {
    o$.subscribe({
      next(value) {
        result.push(value)
      },
      error() {
        result.push(error)
        resolve()
      },
      complete() {
        result.push(complete)
        resolve()
      },
    })
  })

  return result
}

export async function assertObservable<T>(o$: Observable<T>, expected: ResultArray<T>) {
  const actual = await observableToArray(o$)

  assertDeepEqual(actual, expected)
}
