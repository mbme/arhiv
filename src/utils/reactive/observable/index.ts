import { createLogger } from '~/utils/logger'
import { Procedure } from '~/utils/types'
import { noop } from '~/utils/misc'
import { removeAtMut } from '~/utils/array'
import {
  InitCb,
  IObserver,
} from './types'
import { createSubscription } from './subscription'
import { Observer } from './observer'

const log = createLogger('observable')

export class Observable<T> {
  constructor(
    private _init: InitCb<T>,
  ) { }

  subscribe(rawObserver: Partial<IObserver<T>>): Procedure {
    const subscription = createSubscription()
    const observer = new Observer(rawObserver, subscription)

    let destroyCb: Procedure | undefined
    try {
      destroyCb = this._init(observer) || undefined
    } catch (e) {
      log.debug('error on init:', e)
      observer.error(e)
    }

    if (destroyCb) {
      if (observer.isComplete()) {
        destroyCb()
      } else {
        subscription.callbacks.add(destroyCb)
      }
    }

    return subscription
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
    let unsub: Procedure = noop

    return new Observable<K>((observer) => {
      const unsubThis = this.subscribe({
        next: (value) => {
          unsub()

          unsub = map(value).subscribe({
            next: (mappedValue) => observer.next(mappedValue),
            error: observer.error,
          })
        },
        error: (err) => {
          unsub()
          observer.error(err)
        },
        complete: () => {
          unsub()
          observer.complete()
        },
      })

      return () => {
        unsub()
        unsubThis()
      }
    })
  }

  take(limit: number): Observable<T> {
    if (limit < 1) {
      throw new Error('limit must be greater than 0')
    }

    let counter = 0

    return new Observable<T>((observer) => this.subscribe({
      next: (value) => {
        observer.next(value)
        counter += 1

        if (counter === limit) {
          observer.complete()
        }
      },
      error: observer.error,
      complete: observer.complete,
    }))
  }

  buffer(size: number): Observable<T[]> {
    if (size < 1) {
      throw new Error('size must be greater than 0')
    }

    const buffer: T[] = []

    return new Observable<T[]>((observer) => this.subscribe({
      next: (value) => {
        buffer.push(value)

        if (buffer.length > size) {
          removeAtMut(buffer, 0)
        }

        observer.next([...buffer])
      },
      error: observer.error,
      complete: observer.complete,
    }))
  }

  skip(count: number): Observable<T> {
    if (count < 1) {
      throw new Error('count must be greater than 0')
    }

    let skipped = 0

    return new Observable<T>((observer) => this.subscribe({
      next: (value) => {
        if (skipped === count) {
          observer.next(value)
        } else {
          skipped += 1
        }
      },
      error: observer.error,
      complete: observer.complete,
    }))
  }
}
