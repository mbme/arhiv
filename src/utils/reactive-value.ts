import { Counter } from './counter'
import { removeMut } from './index'

type NextCb<T> = (value: T) => void
type ErrorCb = (e: Error) => void
type CompleteCb = () => void
type UnsubscribeCb = () => void

interface IObserver<T> {
  next?: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

export class ReactiveValue<T> {
  private _observers: Array<IObserver<T>> = []
  private _complete = false
  private _nextCounter = new Counter()

  constructor(private _value: T) { }

  get currentValue() {
    return this._value
  }

  private _assertNotComplete() {
    if (this._complete) {
      throw new Error('already complete')
    }
  }

  next(value: T) {
    this._assertNotComplete()
    const callId = this._nextCounter.incAndGet()

    for (const observer of this._observers) {
      if (!observer.next) {
        continue
      }

      observer.next(value)

      // stop iterating if next() was called again
      // so that subscribers wouldn't receive an outdated value
      if (this._nextCounter.value !== callId) {
        return
      }
    }
  }

  error = (e: Error) => {
    this._assertNotComplete()

    for (const observer of this._observers) {
      if (observer.error) {
        observer.error(e)
      }
    }
  }

  complete = () => {
    if (this._complete) {
      return
    }

    for (const observer of this._observers) {
      if (observer.complete) {
        observer.complete()
      }
    }

    this._observers.length = 0
    this._complete = true
  }

  subscribe(observer: IObserver<T>): UnsubscribeCb {
    this._assertNotComplete()

    this._observers.push(observer)

    if (observer.next) {
      observer.next(this._value)
    }

    return () => removeMut(this._observers, observer)
  }

  map<K>(map: (value: T) => K): ReactiveValue<K> {
    const mappedValue = new ReactiveValue<K>(map(this._value))

    this.subscribe({
      next: value => mappedValue.next(map(value)),
      error: mappedValue.error,
      complete: mappedValue.complete,
    })

    return mappedValue
  }

  filter(test: (value: T) => boolean) {
    const mappedValue = new ReactiveValue<T>(this._value)

    this.subscribe({
      next: value => {
        if (test(value)) {
          mappedValue.next(value)
        }
      },
      error: mappedValue.error,
      complete: mappedValue.complete,
    })

    return mappedValue
  }
}
