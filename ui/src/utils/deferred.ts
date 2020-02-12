export class Deferred<T> {
  promise: Promise<T>

  // according to MDN, Promise constructor function is executed immediately,
  // so both resolve and reject would be initialized
  // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise#Parameters
  resolve!: (arg: T) => void
  reject!: (err: Error) => void

  constructor() {
    this.promise = new Promise<T>((resolve, reject) => {
      this.resolve = resolve
      this.reject = reject
    })
  }
}
