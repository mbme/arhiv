import {
  createLogger,
} from '~/logger'
import {
  Cell,
  Observable,
} from '~/reactive'
import { Dict } from '~/utils'

const log = createLogger('web-locks')

export class WebLocks {
  // [lock name]: tab id
  public readonly state: Cell<Dict>

  constructor(
    private _tabId: string,
    private _lockPropName: string,
  ) {
    this.state = new Cell<Dict>(this._read())
    log.info(`tab id: ${_tabId}, lock property: "${_lockPropName}", ${Object.keys(this.state.value).length} active locks`)

    window.addEventListener('storage', this._onStorageUpdate)
    window.addEventListener('beforeunload', this._onBeforeUnload)
  }

  isLocked$(lockName: string): Observable<boolean> {
    return this.state.value$.map(state => !!state[lockName])
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

  private _read(): Dict {
    const valueStr = localStorage.getItem(this._lockPropName)
    if (!valueStr) {
      return {}
    }

    return JSON.parse(valueStr) as Dict
  }

  private _write() {
    localStorage.setItem(this._lockPropName, JSON.stringify(this.state.value))
  }

  private _lock(lockName: string) {
    const state = this.state.value
    if (state[lockName]) {
      throw new Error(`[unreachable] can't acquire lock "${lockName}": already locked`)
    }

    this.state.value = {
      ...state,
      [lockName]: this._tabId,
    }

    this._write()
  }

  private _releaseLock(lockName: string) {
    const state = this.state.value
    if (!state[lockName]) {
      throw new Error(`[unreachable] can't release lock "${lockName}": not locked`)
    }

    if (state[lockName] !== this._tabId) {
      throw new Error(`[unreachable] can't release lock "${lockName}": locked by a different tab`)
    }

    const newState = {
      ...state,
    }
    // tslint:disable-next-line:no-dynamic-delete
    delete newState[lockName]
    this.state.value = newState

    this._write()
  }

  private _onStorageUpdate = (e: StorageEvent) => {
    // key is null on localStorage.clear()
    if (!e.key || e.key === this._lockPropName) {
      this.state.value = this._read()
    }
  }

  private _onBeforeUnload = () => {
    const newState = {
      ...this.state.value,
    }
    let hadActiveLocks = false

    // release all remaining locks
    for (const [lockName, tabId] of Object.entries(newState)) {
      if (tabId === this._tabId) {
        hadActiveLocks = true
        // tslint:disable-next-line:no-dynamic-delete
        delete newState[lockName]
        log.warn(`tab ${tabId} had remaining lock "${lockName}"`)
      }
    }

    if (hadActiveLocks) {
      this.state.value = newState
      this._write()
    }
  }

  destroy() {
    window.removeEventListener('storage', this._onStorageUpdate)
    window.removeEventListener('beforeunload', this._onBeforeUnload)
  }
}
