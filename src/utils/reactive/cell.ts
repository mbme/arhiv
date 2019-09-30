import { Counter } from '../counter'
import { removeMut } from '../array'
import { Observable } from './observable'

type NextCb<T> = (value: T) => void

export class Cell<T> {
  private _subscribers: Array<NextCb<T>> = []
  private _valueCounter = new Counter()

  constructor(private _value: T) { }

  get value() {
    return this._value
  }

  set value(value: T) {
    this._value = value

    const callId = this._valueCounter.incAndGet()
    for (const observer of this._subscribers) {
      observer(value)

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

  next = (value: T) => {
    this.value = value
  }
}
