import { Procedure } from '../types'
import { noop } from '../misc'

type NextCb<T> = (value: T) => void
type ErrorCb = (e: any) => void
type CompleteCb = Procedure
type UnsubscribeCb = Procedure

interface IObserver<T> {
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}

type InitCb<T> = (observer: IObserver<T>) => (Procedure | void)

export class Observable<T> {
  constructor(
    private _init: InitCb<T>,
  ) { }

  subscribe(subscriber: Partial<IObserver<T>>): UnsubscribeCb {
    const destroy = this._init({
      next: subscriber.next || noop,
      error: subscriber.error || noop,
      complete: subscriber.complete || noop,
    }) || noop

    return destroy
  }

  map<K>(map: (value: T) => K): Observable<K> {
    return new Observable<K>((observer) => this.subscribe({
      next: value => observer.next(map(value)),
      error: observer.error,
      complete: observer.complete,
    }))
  }

  tap(cb: (value: T) => void): Observable<T> {
    return this.map((value) => {
      cb(value)

      return value
    })
  }
}
