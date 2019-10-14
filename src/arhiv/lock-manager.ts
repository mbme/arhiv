import {
  createLogger,
} from '~/utils'
import {
  Cell,
  Observable,
} from '~/utils/reactive'

const log = createLogger('isodb:lock-manager')

type State = { type: 'free' }
  | { type: 'db-locked' }
  | { type: 'documents-locked', locks: readonly string[] }

export class LockManager {
  private _state = new Cell<State>({ type: 'free' })

  private _unsub = this._state.value$.subscribe({
    next(currentState) {
      if (currentState.type === 'free') {
        log.info('state -> free')

        return
      }

      if (currentState.type === 'db-locked') {
        log.info('state -> db-locked')

        return
      }

      log.info(`state -> documents locked: ${currentState.locks.join(', ')}`)
    },
  })

  isDocumentLocked$(id: string) {
    return this._state.value$.map((state) => {
      if (state.type === 'free') {
        return false
      }

      if (state.type === 'db-locked') {
        return true
      }

      return state.locks.includes(id)
    })
  }

  private _lockDocument(id: string) {
    const state = this._state.value

    if (state.type === 'db-locked') {
      throw new Error(`[unreachable] can't lock document ${id}: db locked`)
    }

    if (state.type === 'free') {
      this._state.value = { type: 'documents-locked', locks: [id] }

      return
    }

    if (state.locks.includes(id)) {
      throw new Error(`[unreachable] can't lock document ${id}: already locked`)
    }
    this._state.value = { type: 'documents-locked', locks: [...state.locks, id] }
  }

  private _unlockDocument(id: string) {
    const state = this._state.value

    if (state.type === 'free'
      || state.type === 'db-locked'
      || !state.locks.includes(id)
    ) {
      throw new Error(`[unreachable] can't unlock document ${id}: not locked`)
    }

    const locks = state.locks.filter(lock => lock !== id)

    this._state.value = locks.length
      ? { type: 'documents-locked', locks }
      : { type: 'free' }
  }

  acquireDocumentLock$(id: string) {
    return this.isDocumentLocked$(id)
      .filter(isLocked => !isLocked)
      .take(1)
      .switchMap(() => new Observable<void>((observer) => {
        this._lockDocument(id)
        observer.next()

        return () => this._unlockDocument(id)
      }))
  }

  private _lockDB() {
    if (this._state.value.type !== 'free') {
      throw new Error("Can't lock db: not free")
    }

    this._state.value = { type: 'db-locked' }
  }

  private _unlockDB() {
    if (this._state.value.type !== 'db-locked') {
      throw new Error("Can't unlock db: not locked")
    }

    this._state.value = { type: 'free' }
  }

  readonly isDBLocked$ = this._state.value$.map((state) => {
    if (state.type === 'db-locked') {
      return true
    }

    return false
  })

  acquireDBLock$() {
    return this.isDBLocked$
      .filter(isLocked => !isLocked)
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
    this._unsub()
  }
}
