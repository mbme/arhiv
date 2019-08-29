import { Counter } from './counter'
import { Procedure } from './types'
import { noop } from './misc'
import { removeMut } from './array'

type InitCb<T> = (observer: IHotObservable<T>) => (Procedure | void)
type NextCb<T> = (value: T) => void
type ErrorCb = (e: Error) => void
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
}

interface IHotObservable<T> extends IObservable<T> {
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}

export class ReactiveValue<T> implements IHotObservable<T>, IObserver<T> {
  private _subscribers: Array<ISubscriber<T>> = []
  private _complete = false
  private _nextCounter = new Counter()
  private _destroy = noop

  constructor(private _value: T, init?: InitCb<T>) {
    if (init) {
      this._destroy = init(this) || noop
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
  }

  complete = () => {
    if (this._complete) {
      return
    }

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
    this._assertNotComplete()

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

  filter(test: (value: T) => boolean) {
    return new ReactiveValue<T>(
      this._value,
      (observer) => this.subscribe({
        next: value => {
          if (test(value)) {
            observer.next(value)
          }
        },
        error: observer.error,
        complete: observer.complete,
      }),
    )
  }

  take(count: number) {
    if (count < 1) {
      throw new Error(`"count" must be greater than 0, got ${count}`)
    }

    return new ReactiveValue<T>(
      this._value,
      (observer) => {
        let counter = 0

        return this.subscribe({
          next: value => {
            observer.next(value)
            counter += 1
            if (counter === count) {
              observer.complete()
            }
          },
          error: observer.error,
          complete: observer.complete,
        })
      },
    )
  }

  tap(cb: (value: T) => void) {
    return this.map((value) => {
      cb(value)

      return value
    })
  }
}
