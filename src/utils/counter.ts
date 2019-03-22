export class Counter {
  _value = 0

  incAndGet() {
    this._value += 1

    return this._value
  }
}
