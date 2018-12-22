import IsodbReplica from '../core/replica'
import ReplicaInMemStorage from '../core/replica-in-mem-storage'
import PubSub from '../../utils/pubsub'
import LockManager from './lock-manager'
import SyncAgent from './sync-agent'

export default class IsodbClient {
  events = new PubSub()

  _db: IsodbReplica
  _lockManager: LockManager
  _syncAgent: SyncAgent

  constructor() {
    this._db = new IsodbReplica(new ReplicaInMemStorage(this.events))

    this._lockManager = new LockManager()

    this._syncAgent = new SyncAgent(this._db, this._lockManager)
    this._syncAgent.start()
  }

  canLockRecord(id: string) {
    return this._lockManager.canLockRecord(id)
  }
  lockRecord(id: string) {
    return this._lockManager.lockRecord(id)
  }

  async authorize(password: string) {
    const response = await fetch('/api/auth', {
      method: 'post',
      body: password,
    })

    return response.ok
  }

  async deauthorize() {
    document.cookie = 'token=0; path=/'
  }

  destroy() {
    this._syncAgent.stop()
  }
}
