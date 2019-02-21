import IsodbReplica from '~/isodb-core/replica'
import ReplicaInMemStorage from '~/isodb-core/replica-in-mem-storage'
import { createEventsPubSub } from './events'
import LockAgent from './lock-agent'
import SyncAgent from './sync-agent'
import NetworkAgent from './network-agent'
import AuthAgent from './auth-agent'
import { IRecord } from '~/isodb-core/types';

class RecordLock {
  constructor(
    public id: string,
    private _lockAgent: LockAgent,
    private _db: IsodbReplica,
  ) {
    _lockAgent.lockRecord(id);
  }

  updateRecord() {

  }

  deleteRecord() {
    this._db.updateRecord(this.id, { _deleted: true })
    this.release()
  }

  release() {
    this._lockAgent.unlockRecord(this.id)
  }
}

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

  addRecord(record: IRecord) { // FIXME files?
    // TODO lock new record
    if (this.getRecord(record._id)) throw new Error(`can't add record`)
    return this.db.saveRecord(record)
  }

  lockRecord(id: string) {
    return new RecordLock(id, this._lockAgent, this.db)
  }

  getAttachmentUrl(id: string) {
    return this.db.getAttachmentUrl(id)
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
