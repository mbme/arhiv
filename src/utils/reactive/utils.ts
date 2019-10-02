import { Observable } from './observable'
import { Callbacks } from '../component-lifecycle'

export function createInterval$(interval: number) {
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

    return () => callbacks.runAll()
  })
}
