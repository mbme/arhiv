import { createLogger } from '~/logger'
import { ReactiveValue } from '~/utils/reactive'

const log = createLogger('isodb:lock-manager')

type State = { type: 'free' }
  | { type: 'db-locked' }
  | { type: 'documents-locked', locks: readonly string[] }

export class LockManager {
  $state = new ReactiveValue<State>({ type: 'free' })

  constructor() {
    this.$state.subscribe(currentState => {
      if (currentState.type === 'free') {
        log.info('state -> free')

        return
      }

      if (currentState.type === 'db-locked') {
        log.info('state -> db-locked')

        return
      }

      log.info(`state -> documents locked: ${currentState.locks.join(', ')}`)
    })
  }

  isFree() {
    return this.$state.currentValue.type === 'free'
  }

  lockDB() {
    if (this.$state.currentValue.type !== 'free') {
      throw new Error("Can't lock db: not free")
    }

    this.$state.next({ type: 'db-locked' })
  }

  unlockDB() {
    if (this.$state.currentValue.type !== 'db-locked') {
      throw new Error("Can't unlock db: not locked")
    }

    this.$state.next({ type: 'free' })
  }

  $isDocumentLocked(id: string) {
    return this.$state.map((state) => {
      if (state.type === 'free') {
        return false
      }

      if (state.type === 'db-locked') {
        return true
      }

      return state.locks.includes(id)
    })
  }

  addDocumentLock(id: string) {
    const {
      currentValue,
    } = this.$state

    if (currentValue.type === 'free') {
      this.$state.next({ type: 'documents-locked', locks: [id] })

      return
    }

    if (currentValue.type === 'db-locked') {
      throw new Error(`[unreachable] can't lock document ${id}: db locked`)
    }

    if (currentValue.locks.includes(id)) {
      throw new Error(`[unreachable] can't lock document ${id}: already locked`)
    }

    this.$state.next({ type: 'documents-locked', locks: [...currentValue.locks, id] })
  }

  removeDocumentLock(id: string) {
    if (this.$state.currentValue.type === 'free'
      || this.$state.currentValue.type === 'db-locked'
      || !this.$state.currentValue.locks.includes(id)
    ) {
      throw new Error(`[unreachable] can't unlock document ${id}: not locked`)
    }

    const locks = this.$state.currentValue.locks.filter(lock => lock !== id)

    if (locks.length) {
      this.$state.next({ type: 'documents-locked', locks })
    } else {
      this.$state.next({ type: 'free' })
    }
  }

  stop() {
    this.$state.complete()
  }
}
