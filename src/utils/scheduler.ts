import { Procedure } from './types'
import { removeMut } from './array'

type CancellCb = Procedure

export class Scheduler {
  private _timeoutIds: number[] = []

  schedule(task: Procedure, timeout: number): CancellCb {
    const id = window.setTimeout(() => {
      task()
      removeMut(this._timeoutIds, id)
    }, timeout)

    this._timeoutIds.push(id)

    return () => {
      window.clearTimeout(id)
      removeMut(this._timeoutIds, id)
    }
  }

  cancellAll() {
    for (const timeoutId of this._timeoutIds) {
      window.clearTimeout(timeoutId)
    }

    this._timeoutIds.length = 0
  }
}
