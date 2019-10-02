import { Procedure } from './types'

export class Callbacks {
  private _callbacks: Procedure[] = []

  add(...cb: Procedure[]) {
    this._callbacks.push(...cb)
  }

  runAll() {
    for (const callback of this._callbacks) {
      callback()
    }
  }
}
