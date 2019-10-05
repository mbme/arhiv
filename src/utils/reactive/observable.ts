import { Procedure } from '../types'
import { noop } from '../misc'
import { createLogger } from '../logger'
import { removeAtMut } from '../array'
import { Callbacks } from '../component-lifecycle'

const log = createLogger('observable')

type InitCb<T> = (observer: IObserver<T>) => (Procedure | void)
type NextCb<T> = (value: T) => void
type ErrorCb = (e: any) => void
type CompleteCb = Procedure

interface IObserver<T> {
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}

interface ISubscription {
  (): void

  callbacks: Callbacks
}

function createSubscription(): ISubscription {
  const callbacks = new Callbacks()

  const subscription: ISubscription = () => callbacks.runAll()
  subscription.callbacks = callbacks

  return subscription
}

class Subscriber<T> implements IObserver<T> {
  private _complete = false

  constructor(
    private _rawSubscriber: Partial<IObserver<T>>,
    private _subscription: ISubscription,
  ) {
    this._subscription.callbacks.add(() => this._complete = true)
  }

  private _assertNotCompleted() {
    if (this._complete) {
      throw new Error('observable is already complete')
    }
  }

  readonly next = (value: T) => {
    this._assertNotCompleted()

    this._rawSubscriber.next?.(value)
  }

  readonly error = (e: any) => {
    this._assertNotCompleted()

    this._rawSubscriber.error?.(e)
    this._subscription.callbacks.runAll()
  }

  readonly complete = () => {
    this._assertNotCompleted()

    this._rawSubscriber.complete?.()
    this._subscription.callbacks.runAll()
  }
}

export class Observable<T> {
  constructor(
    private _init: InitCb<T>,
  ) { }

  subscribe(rawSubscriber: Partial<IObserver<T>>): Procedure {
    const subscription = createSubscription()
    const subscriber = new Subscriber(rawSubscriber, subscription)

    try {
      subscription.callbacks.add(this._init(subscriber) || noop)
    } catch (e) {
      log.debug('error on init:', e)
      subscriber.error(e)
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
    let unsub: Procedure | undefined

    return new Observable<K>((observer) => this.subscribe({
      next: (value) => {
        unsub?.()

        unsub = map(value).subscribe({
          next: (mappedValue) => observer.next(mappedValue),
          error: observer.error,
        })

        return () => {
          if (!unsub) {
            throw new Error('unreachable: unsub is missing')
          }

          unsub()
        }
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
