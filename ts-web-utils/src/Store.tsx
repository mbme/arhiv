import {
  Cell,
  Observable,
} from '@v/reactive'

export abstract class Store<State> {
  private _cell: Cell<State>

  constructor(initialState: State) {
    this._cell = new Cell(initialState)
  }

  get state$(): Observable<State> {
    return this._cell.value$
  }

  get state(): Readonly<State> {
    return this._cell.value
  }

  protected _setState(newState: State) {
    this._cell.value = newState
  }
}
