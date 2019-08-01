interface IObserver<T> {
  next(value: T): void
  error(error: Error): void
  complete(): void
  closed: boolean
}

interface ICancellable {
  cancel(): void
}

class Observable<T> {
  constructor(subscribe: (observer: IObserver<T>) => ICancellable) { }
}
