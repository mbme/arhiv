import {
  IObserver,
  ISubscription,
} from './types'

export class Observer<T> implements IObserver<T> {
  private _complete = false

  constructor(
    private _rawObserver: Partial<IObserver<T>>,
    private _subscription: ISubscription,
  ) {
    this._subscription.callbacks.add(() => {
      this._complete = true
    })
  }

  private _assertNotCompleted() {
    if (this._complete) {
      throw new Error('observable is already complete')
    }
  }

  isComplete() {
    return this._complete
  }

  readonly next = (value: T) => {
    this._assertNotCompleted()

    if (this._rawObserver.next) {
      this._rawObserver.next(value)
    }
  }

  readonly error = (e: any) => {
    this._assertNotCompleted()

    if (this._rawObserver.error) {
      this._rawObserver.error(e)
    }

    this._subscription.callbacks.runAll(true)
  }

  readonly complete = () => {
    if (this._complete) {
      return
    }

    if (this._rawObserver.complete) {
      this._rawObserver.complete()
    }

    this._subscription.callbacks.runAll(true)
  }
}
