import {
  Counter,
  removeMut,
} from '@v/utils'
import { Observable } from './observable'

type NextCb<T> = (value: T) => void

export class Signal<T = void> {
  private _subscribers: NextCb<T>[] = []
  private _counter = new Counter()

  next(signal: T) {
    const callId = this._counter.incAndGet()

    for (const subscriber of this._subscribers) {
      subscriber(signal)

      // stop iterating if next() was called again
      // so that subscribers wouldn't receive an outdated value
      if (this._counter.value !== callId) {
        return
      }
    }
  }

  readonly signal$ = new Observable<T>((observer) => {
    this._subscribers.push(observer.next)

    return () => removeMut(this._subscribers, observer.next)
  })
}
