import { RecordType } from '~/isodb-core/types'
import {
  IsodbReplica,
  ReplicaInMemStorage,
  Note,
} from './replica'
import { createEventsPubSub } from './events'
import {
  LockAgent,
  SyncAgent,
  NetworkAgent,
  AuthAgent,
} from './agents'

export class IsodbWebClient {
  events = createEventsPubSub()
  db = new IsodbReplica(new ReplicaInMemStorage(), this.events)

  private _networkAgent = new NetworkAgent(this.events)
  private _lockAgent = new LockAgent(this.events)
  private _authAgent = new AuthAgent(this.events, this._networkAgent)
  private _syncAgent = new SyncAgent(this.db, this._lockAgent, this._networkAgent, this._authAgent)

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

  isAuthorized() {
    return this._authAgent.isAuthorized()
  }

  getRecord(id: string) {
    return this.db.getRecord(id)
  }

  getRecords() {
    return this.db.getRecords()
  }

  getNotes(): Note[] {
    return this.getRecords().filter(Note.is)
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
