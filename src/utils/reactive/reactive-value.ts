import {
  HotObservable,
  NextCb,
  ErrorCb,
  CompleteCb,
} from './hot-observable'

export class ReactiveValue<T> extends HotObservable<T> {
  private _value: T

  constructor(initialValue: T) {
    super()
    this._value = initialValue
  }

  get currentValue() {
    return this._value
  }

  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): () => void {
    const unsubscribe = super.subscribe(next, error, complete)

    next(this._value)

    return unsubscribe
  }
}
