import { Counter } from '../counter'
import { noop } from '../misc'
import { removeMut } from '../array'
import {
  IObservable,
  IObserver,
  ISubscriber,
  InitCb,
  UnsubscribeCb,
} from './types'

export class Observable<T> implements IObservable<T>, IObserver<T> {
  private _subscribers: Array<ISubscriber<T>> = []
  private _complete = false
  private _nextCounter = new Counter()
  private _destroy = noop

  constructor(init?: InitCb<T>) {
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

    return () => removeMut(this._subscribers, subscriber)
  }

  map<K>(map: (value: T) => K) {
    return new Observable<K>(
      (observer) => this.subscribe({
        next: value => observer.next(map(value)),
        error: observer.error,
        complete: observer.complete,
      }),
    )
  }

  filter(test: (value: T) => boolean) {
    return new Observable<T>(
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

  take(quantity: number) {
    if (quantity < 1) {
      throw new Error(`"quantity" must be greater than 0, got ${quantity}`)
    }

    return new Observable<T>(
      (observer) => {
        let counter = 0

        return this.subscribe({
          next: value => {
            observer.next(value)
            counter += 1

            if (counter === quantity) {
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
    return new Observable<T>(
      (observer) => this.subscribe({
        next: (value) => {
          cb(value)
          observer.next(value)
        },
        error: observer.error,
        complete: observer.complete,
      }),
    )
  }
}
