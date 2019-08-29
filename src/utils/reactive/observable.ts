import {
  IObserver,
  IObservable,
  InitCb,
  UnsubscribeCb,
} from './types'

export class Observable<T> implements IObservable<T> {
  constructor(private _init: InitCb<T>) { }

  subscribe(observer: IObserver<T>): UnsubscribeCb {
    const destroy = this._init({
      next: observer.next,
      error: (e) => {
        if (observer.error) {
          observer.error(e)
        }

        destroy()
      },
      complete: () => {
        if (observer.complete) {
          observer.complete()
        }

        destroy()
      },
    })

    return destroy
  }
}
