import { Procedure } from './types'
import { removeMut } from './array'

type CancellCb = Procedure

export class Scheduler {
  private _timeoutIds: any[] = []

  schedule(task: Procedure, timeout: number): CancellCb {
    const id = setTimeout(() => {
      task()
      removeMut(this._timeoutIds, id)
    }, timeout)

    this._timeoutIds.push(id)

    return () => {
      clearTimeout(id)
      removeMut(this._timeoutIds, id)
    }
  }

  cancellAll() {
    for (const timeoutId of this._timeoutIds) {
      clearTimeout(timeoutId)
    }

    this._timeoutIds.length = 0
  }
}
