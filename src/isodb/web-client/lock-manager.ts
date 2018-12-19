export default class LockManager {
  _dbLocked = false
  _recordLocks = new Set<string>()

  canLockDB() {
    return !this._dbLocked && this._recordLocks.size === 0
  }
  lockDB() {
    if (!this.canLockDB()) {
      return false
    }

    this._dbLocked = true
    return true
  }
  unlockDB() {
    if (!this._dbLocked) throw new Error("Can't unlock db: not locked")
    this._dbLocked = false
  }

  canLockRecord(id: string) {
    return !this._dbLocked && !this._recordLocks.has(id)
  }
  lockRecord(id: string) {
    if (!this.canLockRecord(id)) {
      return false
    }

    this._recordLocks.add(id)
    return true
  }
  unlockRecord(id: string) {
    if (!this._recordLocks.has(id)) throw new Error(`Can't unlock record ${id}: not locked`)
    this._recordLocks.delete(id)
  }
}
