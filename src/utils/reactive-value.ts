import { Counter } from './counter'
import { noop } from './misc'
import { removeMut } from './array'
import { Procedure } from './types'

type InitCb<T> = (observer: IObservable<T>) => (Procedure | void)
type NextCb<T> = (value: T) => void
type ErrorCb = (e: any) => void
type CompleteCb = () => void
type UnsubscribeCb = () => void

interface ISubscriber<T> {
  next?: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

interface IObserver<T> extends ISubscriber<T> {
  next: NextCb<T>
}

interface IObservable<T> {
  subscribe(subscriber: ISubscriber<T>): UnsubscribeCb
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}

export class ReactiveValue<T> implements IObservable<T>, IObserver<T> {
  private _subscribers: Array<ISubscriber<T>> = []
  private _complete = false
  private _nextCounter = new Counter()
  private _destroy = noop

  constructor(private _value: T, init?: InitCb<T>) {
    if (!init) {
      return
    }

    try {
      this._destroy = init(this) || noop

      // make sure we call destroy function if observable was completed during init
      if (this._complete) {
        this._destroy()
      }
    } catch (e) {
      // tslint:disable-next-line:no-unsafe-any
      this.error(e)
    }
  }

  get currentValue() {
    return this._value
  }

  private _assertNotComplete() {
    if (this._complete) {
      throw new Error('already complete')
    }
  }

  next = (value: T) => {
    this._assertNotComplete()

    const callId = this._nextCounter.incAndGet()
    this._value = value

    for (const observer of this._subscribers) {
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

    for (const observer of this._subscribers) {
      if (observer.error) {
        observer.error(e)
      }
    }
    this._subscribers.length = 0

    this._destroy()
    this._complete = true
  }

  complete = () => {
    this._assertNotComplete()

    for (const observer of this._subscribers) {
      if (observer.complete) {
        observer.complete()
      }
    }
    this._subscribers.length = 0

    this._destroy()
    this._complete = true
  }

  subscribe(subscriber: ISubscriber<T>): UnsubscribeCb {
    this._subscribers.push(subscriber)

    if (subscriber.next) {
      subscriber.next(this._value)
    }

    return () => removeMut(this._subscribers, subscriber)
  }

  map<K>(map: (value: T) => K) {
    return new ReactiveValue<K>(
      map(this._value),
      (observer) => this.subscribe({
        next: value => observer.next(map(value)),
        error: observer.error,
        complete: observer.complete,
      }),
    )
  }

  tap(cb: (value: T) => void) {
    this.subscribe({
      next: cb,
    })

    return this
  }
}
