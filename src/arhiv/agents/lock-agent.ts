import { createLogger } from '~/logger'
import { WebClientEvents } from '../events'

const log = createLogger('isodb-lock-agent')

interface IFree {
  state: 'free'
}

interface IDBLocked {
  state: 'db-locked'
}

interface IRecordsLocked {
  state: 'records-locked'
  records: Set<string>
}

type State = IFree | IDBLocked | IRecordsLocked

export class LockAgent {
  state: State = { state: 'free' }

  constructor(public events: WebClientEvents) { }

  private _notify() {
    const dbLocked = this.state.state === 'db-locked'
    const recordsLocked = this.state.state === 'records-locked' ? this.state.records : new Set<string>()
    this.events.emit('isodb-lock', [dbLocked, recordsLocked])
    log.info(`state -> ${this.state.state}${recordsLocked.size ? ' ' + Array.from(recordsLocked).join(', ') : ''}`)
  }

  isFree() {
    return this.state.state === 'free'
  }

  lockDB() {
    if (!this.isFree()) {
      throw new Error("Can't lock db: not free")
    }

    this.state = {
      state: 'db-locked',
    }

    this._notify()
  }

  unlockDB() {
    if (this.state.state !== 'db-locked') {
      throw new Error("Can't unlock db: not locked")
    }

    this.state = {
      state: 'free',
    }

    this._notify()
  }

  isRecordLocked(id: string) {
    if (this.state.state === 'db-locked') {
      return true
    }

    if (this.state.state === 'records-locked' && this.state.records.has(id)) {
      return true
    }

    return false
  }

  lockRecord(id: string) {
    if (this.state.state === 'db-locked') {
      throw new Error(`Can't lock record ${id}: db locked`)
    }

    if (this.state.state === 'free') {
      this.state = {
        state: 'records-locked',
        records: new Set([id]),
      }

      this._notify()

      return
    }

    if (this.state.records.has(id)) {
      throw new Error(`Can't lock record ${id}: already locked`)
    }

    this.state.records.add(id)

    this._notify()
  }

  unlockRecord(id: string) {
    if (this.state.state !== 'records-locked' || !this.state.records.has(id)) {
      throw new Error(`Can't unlock record ${id}: not locked`)
    }

    this.state.records.delete(id)

    if (!this.state.records.size) {
      this.state = {
        state: 'free',
      }
    }

    this._notify()
  }
}
