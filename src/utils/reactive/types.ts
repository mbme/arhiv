import { Procedure } from '../types'

export type InitCb<T> = (observer: IObservable<T>) => (Procedure | void)
export type NextCb<T> = (value: T) => void
export type ErrorCb = (e: any) => void
export type CompleteCb = () => void
export type UnsubscribeCb = () => void

export interface ISubscriber<T> {
  next?: NextCb<T>
  error?: ErrorCb
  complete?: CompleteCb
}

export interface IObserver<T> extends ISubscriber<T> {
  next: NextCb<T>
}

export interface IObservable<T> {
  subscribe(subscriber: ISubscriber<T>): UnsubscribeCb
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}
