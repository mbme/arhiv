import { removeMut } from '../index'

export type NextCb<T> = (value: T) => void
export type ErrorCb = (e: Error) => void
export type CompleteCb = () => void

type DestroyCb = () => void
export type InitCb<T> = (next: NextCb<T>, error: ErrorCb, complete: CompleteCb) => DestroyCb | undefined

export type UnsubscribeCb = () => void

interface IObserver<T> {
  next: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

// push-based "hot" observable
export class HotObservable<T> {
  private _observers: Array<IObserver<T>> = []
  private _complete = false
  private _destroyCb?: DestroyCb

  constructor(init?: InitCb<T>) {
    if (init) {
      this._destroyCb = init(this.next, this.error, this.complete)
    }
  }

  private assertNotComplete() {
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
    this.assertNotComplete()

    for (const observer of this._observers) {
      observer.next(value)
    }
  }

  error = (e: Error) => {
    this.assertNotComplete()

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
    this.assertNotComplete()

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
