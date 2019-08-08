export class Counter {
  value = 0

  incAndGet() {
    this.value += 1

    return this.value
  }
}
