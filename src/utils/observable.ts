import { removeMut } from './index'

type Sub<T> = (value: T) => void

export default class Observable<T> {
  _subs: Array<Sub<T>> = []

  constructor(public _value: T) { }

  get value() {
    return this._value
  }

  set value(newValue: T) {
    this._value = newValue
    this._subs.forEach(sub => sub(newValue))
  }

  on(sub: Sub<T>) {
    this._subs.push(sub)

    return () => this.off(sub)
  }

  off(sub: Sub<T>) {
    removeMut(this._subs, sub)
  }
}
