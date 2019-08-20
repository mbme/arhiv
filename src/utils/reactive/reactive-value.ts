import {
  NextCb,
  ErrorCb,
  CompleteCb,
  UnsubscribeCb,
  InitCb,
} from './types'
import {
  HotObservable,
} from './hot-observable'

export class ReactiveValue<T> extends HotObservable<T> {
  private _value: T
  private _initSubscriptionTimeoutId?: number

  constructor(initialValue: T, init?: InitCb<T>) {
    super(init)
    this._value = initialValue
  }

  next(value: T) {
    if (value !== this._value) {
      this._value = value
      super.next(value)
      clearTimeout(this._initSubscriptionTimeoutId)
    }
  }

  get currentValue() {
    return this._value
  }

  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): UnsubscribeCb {
    const unsubscribe = super.subscribe(next, error, complete)

    // asynchronously call next() to handle the case
    // when unsubscribe() is called immediately in the next()
    // which results into ReferenceError
    this._initSubscriptionTimeoutId = window.setTimeout(() => {
      next(this._value)
    }, 0)

    return () => {
      clearTimeout(this._initSubscriptionTimeoutId)
      unsubscribe()
    }
  }

  map<K>(map: (value: T) => K): ReactiveValue<K> {
    return new ReactiveValue(map(this._value), (next, error, complete) => {
      const unsubscribe = this.subscribe(
        value => next(map(value)),
        error,
        complete,
      )

      return unsubscribe
    })
  }
}
