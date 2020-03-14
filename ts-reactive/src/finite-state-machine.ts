import { createLogger } from '@v/logger'
import { Cell } from './cell'

const log = createLogger('state-machine')

type FSM<States extends string, Events extends string> = {
  [key in States]: {
    [event in Events]?: States
  }
}

export class FiniteStateMachine<States extends string, Events extends string> {
  state$: Cell<States>

  constructor(
    initialState: States,
    private _fsm: FSM<States, Events>,
  ) {
    this.state$ = new Cell<States>(initialState)
  }

  dispatchEvent(event: Events) {
    const currentState = this.state$.value

    const newState: States | undefined = this._fsm[currentState][event]

    if (!newState) {
      log.warn(`ignoring unexpected event ${event} in state ${currentState}`)

      return
    }

    this.state$.value = newState
  }
}
