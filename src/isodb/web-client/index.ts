import IsodbReplica from '../core/replica'
import ReplicaInMemStorage from '../core/replica-in-mem-storage'
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

  lockRecord(id: string) {
    this._lockAgent.lockRecord(id)
  }

  async authorize(password: string) {
    return this._networkAgent.authorize(password)
  }

  deauthorize() {
    this._networkAgent.deauthorize()
  }

  syncNow() {
    this._syncAgent.syncNow()
  }
}
