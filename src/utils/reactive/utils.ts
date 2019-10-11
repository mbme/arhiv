import { Observable } from './observable'
import { Callbacks } from '../callbacks'

export function interval$(interval: number) {
  return new Observable<undefined>((observer) => {
    const intervalId = setInterval(observer.next, interval)

    return () => clearInterval(intervalId)
  })
}

export function merge$<T>(...observables: Array<Observable<T>>) {
  return new Observable<T>((observer) => {
    const callbacks = new Callbacks()

    for (const observable of observables) {
      callbacks.add(observable.subscribe(observer))
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
