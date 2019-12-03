import {
  createLogger,
} from '~/logger'
import {
  Cell,
  Observable,
} from '~/reactive'

const log = createLogger('web-locks')

interface ILocks {
  // lock name : tab id
  [key: string]: string | undefined
}

export class WebLocks {
  private _state: Cell<ILocks>

  constructor(
    private _tabId: string,
    private _lockPropName: string,
  ) {
    this._state = new Cell<ILocks>(this._read())
    log.info(`tab id: ${_tabId}, lock property: "${_lockPropName}", ${Object.keys(this._state.value).length} active locks`)

    window.addEventListener('storage', this._onStorageUpdate)
    window.addEventListener('beforeunload', this._onBeforeUnload)
  }

  isLocked$(lockName: string): Observable<boolean> {
    return this._state.value$.map(state => !!state[lockName])
  }

  acquireLock$(lockName: string) {
    return this.isLocked$(lockName)
      .filter((isLocked): isLocked is false => !isLocked)
      .take(1)
      .switchMap(() => new Observable<void>((observer) => {
        this._lock(lockName)
        observer.next()

        return () => this._releaseLock(lockName)
      }))
  }

  private _read(): ILocks {
    const valueStr = localStorage.getItem(this._lockPropName)
    if (!valueStr) {
      return {}
    }

    return JSON.parse(valueStr) as ILocks
  }

  private _write() {
    localStorage.setItem(this._lockPropName, JSON.stringify(this._state.value))
  }

  private _lock(lockName: string) {
    const state = this._state.value
    if (state[lockName]) {
      throw new Error(`[unreachable] can't acquire lock "${lockName}": already locked`)
    }

    this._state.value = {
      ...state,
      [lockName]: this._tabId,
    }

    this._write()
  }

  private _releaseLock(lockName: string) {
    const state = this._state.value
    if (!state[lockName]) {
      throw new Error(`[unreachable] can't release lock "${lockName}": not locked`)
    }

    if (state[lockName] !== this._tabId) {
      throw new Error(`[unreachable] can't release lock "${lockName}": locked by a different tab`)
    }

    this._state.value = {
      ...state,
      [lockName]: undefined,
    }

    this._write()
  }

  private _onStorageUpdate = (e: StorageEvent) => {
    // key is null on localStorage.clear()
    if (!e.key || e.key === this._lockPropName) {
      this._state.value = this._read()
    }
  }

  private _onBeforeUnload = () => {
    const newState = {
      ...this._state.value,
    }
    let hadActiveLocks = false

    // release all remaining locks
    for (const [lockName, tabId] of Object.entries(newState)) {
      if (tabId === this._tabId) {
        hadActiveLocks = true
        newState[lockName] = undefined
        log.warn(`tab ${tabId} had remaining lock "${lockName}"`)
      }
    }

    if (hadActiveLocks) {
      this._state.value = newState
      this._write()
    }
  }

  destroy() {
    window.removeEventListener('storage', this._onStorageUpdate)
    window.removeEventListener('beforeunload', this._onBeforeUnload)
  }
}
