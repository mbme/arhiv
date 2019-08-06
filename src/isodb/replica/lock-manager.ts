import { createLogger } from '~/logger'
import { ReactiveValue } from '~/utils/reactive'

const log = createLogger('arhiv:lock-manager')

type DocumentLocks = readonly string[]
type State = 'free' | 'db-locked' | DocumentLocks

export interface ILock {
  release(): void
}

export class LockManager {
  $state = new ReactiveValue<State>('free')

  constructor() {
    this.$state.subscribe(
      currentState => {
        if (currentState === 'free') {
          log.info('state -> free')
        } else if (currentState === 'db-locked') {
          log.info('state -> db-locked')
        } else {
          log.info(`state -> documents locked: ${currentState.join(', ')}`)
        }
      },
    )
  }

  isFree() {
    return this.$state.currentValue === 'free'
  }

  lockDB() { // FIXME same as document lock
    if (!this.isFree()) {
      throw new Error("Can't lock db: not free")
    }

    this.$state.next('db-locked')
  }

  unlockDB() {
    if (this.$state.currentValue !== 'db-locked') {
      throw new Error("Can't unlock db: not locked")
    }

    this.$state.next('free')
  }

  isDocumentLocked(id: string) {
    if (this.$state.currentValue === 'db-locked') {
      return true
    }

    if (this.$state.currentValue === 'free') {
      return false
    }

    return this.$state.currentValue.includes(id)
  }

  private _acquireLock(id: string) {
    return {
      release: () => {
        if (this.$state.currentValue === 'free'
          || this.$state.currentValue === 'db-locked'
          || !this.$state.currentValue.includes(id)
        ) {
          throw new Error(`[unreachable] can't unlock document ${id}: not locked`)
        }

        const locks = this.$state.currentValue.filter(lock => lock !== id)

        if (locks.length) {
          this.$state.next(locks)
        } else {
          this.$state.next('free')
        }
      },
    }
  }

  lockDocument(id: string): Promise<ILock> {
    if (this.$state.currentValue === 'free') {
      this.$state.next([id])

      return Promise.resolve(this._acquireLock(id))
    }

    return new Promise((resolve, reject) => {
      const unsubscribe = this.$state.subscribe(
        () => {
          if (this.isDocumentLocked(id)) {
            return
          }

          unsubscribe()

          this.$state.next([...this.$state.currentValue, id])
          resolve(this._acquireLock(id))
        },
        reject,
        reject,
      )
    })
  }

  stop() {
    this.$state.complete()
  }
}
