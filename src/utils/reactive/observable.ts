import {
  IObservable,
  InitCb,
  NextCb,
  ErrorCb,
  CompleteCb,
  UnsubscribeCb,
} from './types'

export class Observable<T> implements IObservable<T> {
  constructor(private _init: InitCb<T>) { }

  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): UnsubscribeCb {
    const destroy = this._init(
      next,
      (e) => {
        if (error) {
          error(e)
        }

        destroy()
      },
      () => {
        if (complete) {
          complete()
        }

        destroy()
      },
    )

    return destroy
  }
}
