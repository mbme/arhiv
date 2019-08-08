import { createLogger } from '~/logger'
import { ReactiveValue } from './reactive-value'

const log = createLogger('state-machine')

export class FiniteStateMachine<States, Events extends string> {
  $state: ReactiveValue<States>

  constructor(
    initialState: States,
    private _transition: (currentState: States, events: Events) => States,
  ) {
    this.$state = new ReactiveValue(initialState)
  }

  dispatchEvent(event: Events) {
    const currentState = this.$state.currentValue
    this.$state.next(this._transition(currentState, event))

    if (this.$state.currentValue === currentState) {
      log.debug(`ignored event ${event}, current state: ${currentState}`)
    }
  }
}
