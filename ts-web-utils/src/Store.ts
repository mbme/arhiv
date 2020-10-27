import {
  Cell,
} from '@v/reactive'
import { Obj } from '@v/utils'

export abstract class Store<State extends Obj> {
  private _cell: Cell<State>

  constructor(initialState: State) {
    this._cell = new Cell(initialState)
  }

  get state$(): Cell<State> {
    return this._cell
  }

  get state(): Readonly<State> {
    return this._cell.value
  }

  protected _setState(newState: Partial<State>) {
    this._cell.value = {
      ...this._cell.value,
      ...newState,
    }
  }
}
