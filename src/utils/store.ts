import Observable from './observable'

export default class Store<S extends {}> {
  $state: Observable<S>

  get state() {
    return this.$state.value
  }

  setState(newState: Partial<S>) {
    this.$state.value = {
      ...this.$state.value,
      ...newState,
    }
  }

  constructor(initialState: S) {
    this.$state = new Observable<S>(initialState)
  }
}
