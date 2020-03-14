import {
  noop,
  Callbacks,
} from '@v/utils'
import { Observable } from './observable'

export function interval$(interval: number) {
  return new Observable<undefined>((observer) => {
    const intervalId = setInterval(observer.next, interval)

    return () => clearInterval(intervalId)
  })
}

export function merge$<T>(...observables: Observable<T>[]) {
  return new Observable<T>((observer) => {
    const callbacks = new Callbacks()

    let completeCount = 0
    for (const observable of observables) {
      callbacks.add(observable.subscribe({
        next: observer.next,
        error: observer.error,
        complete() { // eslint-disable-line no-loop-func
          completeCount += 1
          if (completeCount === observables.length) {
            observer.complete()
          }
        },
      }))
    }

    return () => callbacks.runAll(true)
  })
}

export function blobUrl$(blob: Blob) {
  return new Observable<string>((observer) => {
    const url = URL.createObjectURL(blob)
    observer.next(url)

    return () => URL.revokeObjectURL(url)
  })
}

export function promise$<T>(promise: Promise<T>): Observable<T> {
  return new Observable<T>((observer) => {
    let completed = false

    promise.then(
      (value) => {
        if (!completed) {
          observer.next(value)
          observer.complete()
        }
      },
      (err) => {
        if (!completed) {
          observer.error(err)
        }
      },
    )

    return () => {
      completed = true
    }
  })
}

export function of$<T>(value: T): Observable<T> {
  return new Observable<T>((observer) => {
    observer.next(value)
    observer.complete()

    return noop
  })
}

export function zip$<T>(...observables: Observable<T>[]) {
  return new Observable<ReadonlyArray<T | undefined>>((observer) => {
    const callbacks = new Callbacks()

    const state = new Array<T | undefined>(observables.length)
    let completeCount = 0

    for (let i = 0; i < observables.length; i += 1) {
      const observable = observables[i]

      callbacks.add(observable.subscribe({
        next(value) {
          state[i] = value
          observer.next(state)
        },
        error: observer.error,
        complete() { // eslint-disable-line no-loop-func
          completeCount += 1
          if (completeCount === observables.length) {
            observer.complete()
          }
        },
      }))
    }

    return () => callbacks.runAll(true)
  })
}
