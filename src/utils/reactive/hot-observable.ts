import {
  removeMut,
  noop,
} from '../index'
import {
  IHotObservable,
  IObserver,
  NextCb,
  ErrorCb,
  CompleteCb,
  UnsubscribeCb,
  DestroyCb,
  InitCb,
} from './types'

// push-based "hot" lazy observable
export class HotObservable<T> implements IHotObservable<T> {
  private _observers: Array<IObserver<T>> = []
  private _complete = false
  private _destroyCb: DestroyCb = noop

  constructor(private _init?: InitCb<T>) { }

  private _assertNotComplete() {
    if (this._complete) {
      throw new Error('already complete')
    }
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

    this._destroyCb()

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

    // init datasource on first subscriber
    if (this._observers.length === 1 && this._init) {
      this._destroyCb = this._init(this.next, this.error, this.complete)
    }

    return () => {
      if (complete) {
        complete()
      }
      removeMut(this._observers, observer)

      // destroy datasource if no more subscribers
      if (this._observers.length === 0) {
        this._destroyCb()
      }
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
