import { removeMut } from '../index'
import {
  IObservable,
  IObserver,
  NextCb,
  ErrorCb,
  CompleteCb,
  UnsubscribeCb,
} from './types'

export type DestroyCb = () => void
export type InitCb<T> = (next: NextCb<T>, error: ErrorCb, complete: CompleteCb) => DestroyCb | undefined

// push-based "hot" observable
export class HotObservable<T> implements IObservable<T> {
  private _observers: Array<IObserver<T>> = []
  private _complete = false
  private _destroyCb?: DestroyCb

  constructor(init?: InitCb<T>) {
    if (init) {
      this._destroyCb = init(this.next, this.error, this.complete)
    }
  }

  private _assertNotComplete() {
    if (this._complete) {
      throw new Error('already complete')
    }
  }

  destroy() {
    if (this._destroyCb) {
      this._destroyCb()
    }
    this.complete()
  }

  next = (value: T) => {
    this._assertNotComplete()

    for (const observer of this._observers) {
      observer.next(value)
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

  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): UnsubscribeCb {
    this._assertNotComplete()

    const observer = {
      next,
      error,
      complete,
    }
    this._observers.push(observer)

    return () => {
      if (complete) {
        complete()
      }
      removeMut(this._observers, observer)
    }
  }

  map<K>(map: (value: T) => K): HotObservable<K> {
    return new HotObservable((next, error, complete) => {
      const unsubscribe = this.subscribe(
        value => next(map(value)),
        error,
        complete,
      )

      return unsubscribe
    })
  }
}
