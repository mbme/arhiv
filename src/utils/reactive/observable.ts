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
    const observer = {
      next: subscriber.next || noop,
      error: subscriber.error || noop,
      complete: subscriber.complete || noop,
    }

    try {
      return this._init(observer) || noop
    } catch (e) {
      console.error(e)
      observer.error(e)

      return noop
    }
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

  filter(filter: (value: T) => boolean) {
    return new Observable<T>((observer) => this.subscribe({
      next: (value) => {
        if (filter(value)) {
          observer.next(value)
        }
      },
      error: observer.error,
      complete: observer.complete,
    }))
  }

  switchMap<K>(map: (value: T) => Observable<K>): Observable<K> {
    let unsub: Procedure | undefined

    return new Observable<K>((observer) => this.subscribe({
      next: (value) => {
        if (unsub) {
          unsub()
        }

        unsub = map(value).subscribe({
          next: (mappedValue) => observer.next(mappedValue),
          error: observer.error,
          complete: observer.complete,
        })

        return unsub
      },
      error: observer.error,
      complete: observer.complete,
    }))
  }

  take(limit: number): Observable<T> {
    if (limit < 1) {
      throw new Error('limit must be greater than 0')
    }

    let counter = 0

    return new Observable<T>((observer) => {
      const unsub = this.subscribe({
        next: (value) => {
          observer.next(value)
          counter += 1

          if (counter === limit) {
            observer.complete()
            unsub()
          }
        },
        error: observer.error,
        complete: observer.complete,
      })

      return unsub
    })
  }
}
