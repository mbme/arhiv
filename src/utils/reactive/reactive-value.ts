import {
  HotObservable,
  NextCb,
  ErrorCb,
  CompleteCb,
  InitCb,
  UnsubscribeCb,
} from './hot-observable'

export class ReactiveValue<T> extends HotObservable<T> {
  private _value: T

  constructor(initialValue: T, init?: InitCb<T>) {
    super(init)
    this._value = initialValue
  }

  next = (value: T) => {
    if (value !== this._value) {
      super.next(value)
    }
  }

  get currentValue() {
    return this._value
  }

  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): UnsubscribeCb {
    const unsubscribe = super.subscribe(next, error, complete)

    next(this._value)

    return unsubscribe
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
