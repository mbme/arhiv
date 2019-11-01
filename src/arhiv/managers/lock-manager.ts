import {
  createLogger,
  Callbacks,
} from '~/utils'
import {
  Cell,
  Observable,
} from '~/utils/reactive'

const log = createLogger('isodb:lock-manager')

type State = 'db-locked' | readonly string[]

export class LockManager {
  private _state = new Cell<State>([])
  private _callbacks = new Callbacks()

  constructor() {
    this._callbacks.add(
      this._state.value$.subscribe({
        next(currentState) {
          if (currentState === 'db-locked') {
            log.debug('lock state -> db-locked')

            return
          }

          if (currentState.length === 0) {
            log.debug('lock state -> free')

            return
          }

          log.debug(`lock state -> documents locked: ${currentState.join(', ')}`)
        },
      }),
    )
  }

  isDocumentLocked$(id: string) {
    return this._state.value$.map((state) => {
      if (state === 'db-locked') {
        return true
      }

      return state.includes(id)
    })
  }

  private _lockDocument(id: string) {
    const state = this._state.value

    if (state === 'db-locked') {
      throw new Error(`[unreachable] can't lock document ${id}: db locked`)
    }

    if (state.includes(id)) {
      throw new Error(`[unreachable] can't lock document ${id}: already locked`)
    }

    this._state.value = [...state, id]
  }

  private _unlockDocument(id: string) {
    const state = this._state.value

    if (state === 'db-locked') {
      throw new Error(`[unreachable] can't unlock document ${id}: db locked`)
    }

    if (!state.includes(id)) {
      throw new Error(`[unreachable] can't unlock document ${id}: not locked`)
    }

    this._state.value = state.filter(lock => lock !== id)
  }

  acquireDocumentLock$(id: string) {
    return this.isDocumentLocked$(id)
      .filter((isLocked): isLocked is false => !isLocked)
      .take(1)
      .switchMap(() => new Observable<void>((observer) => {
        this._lockDocument(id)
        observer.next()

        return () => this._unlockDocument(id)
      }))
  }

  private _lockDB() {
    if (this._state.value === 'db-locked') {
      throw new Error("Can't lock db: its already locked")
    }

    if (this._state.value.length) {
      throw new Error("Can't lock db: some documents are locked")
    }

    this._state.value = 'db-locked'
  }

  private _unlockDB() {
    if (this._state.value !== 'db-locked') {
      throw new Error("Can't unlock db: not locked")
    }

    this._state.value = []
  }

  isDBLocked() {
    return this._state.value === 'db-locked'
  }

  acquireDBLock$() {
    return this._state.value$.map((state) => {
      if (state === 'db-locked') {
        return false
      }

      return state.length === 0
    })
      .filter(isFree => isFree)
      .take(1)
      .switchMap(() => new Observable<void>((observer) => {
        this._lockDB()
        observer.next()

        return () => {
          this._unlockDB()
        }
      }))
  }

  stop() {
    this._callbacks.runAll(true)
  }
}
