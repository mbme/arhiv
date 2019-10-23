import {
  Procedure,
  AsyncProcedure,
} from './types'

export class Callbacks {
  private _callbacks: Procedure[] = []

  add(...cb: Procedure[]) {
    this._callbacks.push(...cb)
  }

  runAll(clear = false) {
    for (const callback of this._callbacks) {
      callback()
    }

    if (clear) {
      this.clear()
    }
  }

  clear() {
    this._callbacks = []
  }
}

export class AsyncCallbacks {
  private _callbacks: AsyncProcedure[] = []

  add(...cb: AsyncProcedure[]) {
    this._callbacks.push(...cb)
  }

  async runAll(clear = false, reverseOrder = false) {
    const items = reverseOrder ? [...this._callbacks].reverse() : this._callbacks

    for (const callback of items) {
      await callback()
    }

    if (clear) {
      this.clear()
    }
  }

  clear() {
    this._callbacks = []
  }
}
