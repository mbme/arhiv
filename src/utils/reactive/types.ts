export type NextCb<T> = (value: T) => void
export type ErrorCb = (e: Error) => void
export type CompleteCb = () => void

export type DestroyCb = () => void
export type InitCb<T> = (next: NextCb<T>, error: ErrorCb, complete: CompleteCb) => DestroyCb

export type SubscribeCb<T> = (next: NextCb<T>, error?: ErrorCb, complete?: CompleteCb) => UnsubscribeCb
export type UnsubscribeCb = () => void

export interface IObserver<T> {
  next: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

export interface IObservable<T> {
  subscribe: SubscribeCb<T>
}

export interface IHotObservable<T> extends IObservable<T> {
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}
