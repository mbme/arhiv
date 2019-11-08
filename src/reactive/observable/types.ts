import { Procedure } from '~/utils/types'
import { Callbacks } from '~/utils/callbacks'

export type InitCb<T> = (observer: IObserver<T>) => (Procedure | void)
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
