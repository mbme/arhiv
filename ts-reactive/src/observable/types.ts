import {
  Procedure,
  Callbacks,
} from '@v/utils'

export type InitCb<T> = (observer: IObserver<T>) => Procedure
export type NextCb<T> = (value: T) => void
export type ErrorCb = (e: any) => void
export type CompleteCb = Procedure

export interface IObserver<T> {
  next: NextCb<T>
  error: ErrorCb
  complete: CompleteCb
}

export interface ISubscription {
  (): void

  callbacks: Callbacks
}
