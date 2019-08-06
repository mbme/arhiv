export type NextCb<T> = (value: T) => void
export type ErrorCb = (e: Error) => void
export type CompleteCb = () => void

export type UnsubscribeCb = () => void

export interface IObserver<T> {
  next: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

export interface IObservable<T> {
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
  subscribe(next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb): UnsubscribeCb
}
