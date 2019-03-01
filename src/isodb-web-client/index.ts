import { RecordType } from '~/isodb-core/types'
import IsodbReplica from './replica'
import ReplicaInMemStorage from './replica-in-mem-storage'
import { createEventsPubSub } from './events'
import LockAgent from './lock-agent'
import SyncAgent from './sync-agent'
import NetworkAgent from './network-agent'
import AuthAgent from './auth-agent'

export default class IsodbClient {
  events = createEventsPubSub()

  db = new IsodbReplica(new ReplicaInMemStorage(), this.events)

  _networkAgent = new NetworkAgent(this.events)
  _lockAgent = new LockAgent(this.events)
  _authAgent = new AuthAgent(this.events, this._networkAgent)
  _syncAgent = new SyncAgent(this.db, this._lockAgent, this._networkAgent, this._authAgent)

  start() {
    this._networkAgent.start()
    this._authAgent.start()
    this._syncAgent.start()
  }

  stop() {
    this._networkAgent.stop()
    this._authAgent.stop()
    this._syncAgent.stop()
  }

  getRecord(id: string) {
    return this.db.getRecord(id)
  }

  getRecords() {
    return this.db.getRecords()
  }

  createRecord(recordType: RecordType) {
    return this.db.createRecord(recordType)
  }

  lockRecord(id: string) {
    this._lockAgent.lockRecord(id)
  }

  release(id: string) {
    this._lockAgent.unlockRecord(id)
  }

  async authorize(password: string) {
    return this._authAgent.authorize(password)
  }

  deauthorize() {
    this._authAgent.deauthorize()
  }

  syncNow() {
    this._syncAgent.syncNow()
  }
}
