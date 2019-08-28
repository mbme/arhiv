import { createLogger } from '~/logger'
import { ReactiveValue } from './reactive/reactive-value'

const log = createLogger('state-machine')

type FSM<States extends string, Events extends string> = {
  [key in States]: {
    [event in Events]?: States
  }
}

export class FiniteStateMachine<States extends string, Events extends string> {
  $state: ReactiveValue<States>

  constructor(
    initialState: States,
    private _fsm: FSM<States, Events>,
  ) {
    this.$state = new ReactiveValue<States>(initialState)
  }

  dispatchEvent(event: Events) {
    const currentState = this.$state.currentValue

    const newState: States | undefined = this._fsm[currentState][event]

    if (!newState) {
      log.warn(`ignoring unexpected event ${event} in state ${currentState}`)

      return
    }

    this.$state.next(newState)
  }
}
