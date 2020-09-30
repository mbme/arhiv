import {
  Counter,
  removeMut,
} from '@v/utils'
import { Observable } from './observable'

type NextCb<T> = (value: T) => void

export class Cell<T> {
  private _subscribers: NextCb<T>[] = []
  private _valueCounter = new Counter()

  constructor(
    private _value: T,
    private _distinctUntilChanged = false,
  ) { }

  get value() {
    return this._value
  }

  set value(value: T) {
    if (this._distinctUntilChanged && value === this._value) {
      return
    }

    this._value = value

    const callId = this._valueCounter.incAndGet()
    for (const subscriber of this._subscribers) {
      subscriber(value)

      // stop iterating if next() was called again
      // so that subscribers wouldn't receive an outdated value
      if (this._valueCounter.value !== callId) {
        return
      }
    }
  }

  readonly value$ = new Observable<T>((observer) => {
    this._subscribers.push(observer.next)

    observer.next(this._value)

    return () => removeMut(this._subscribers, observer.next)
  })
}
