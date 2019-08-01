import { removeMut } from './index'

type NextCb<T> = (value: T) => void
type ErrorCb = (e: Error) => void
type CompleteCb = () => void

type DestroyCb = () => void
type InitCb<T> = (next: NextCb<T>, error: ErrorCb, complete: CompleteCb) => DestroyCb | undefined

interface IObserver<T> {
  next: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

export class ReactiveValue<T> {
  private _observers: Array<IObserver<T>> = []
  private _complete = false
  private _destroyCb?: DestroyCb

  constructor(init: InitCb<T>) {
    this._destroyCb = init(this.next, this.error, this.complete)
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

  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): () => void {
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

  map<K>(map: (value: T) => K): ReactiveValue<K> {
    return new ReactiveValue((next, error, complete) => {
      this.subscribe(
        value => next(map(value)),
        error,
        complete,
      )

      return this._destroyCb
    })
  }
}
