import IsodbReplica from '../core/replica'
import ReplicaInMemStorage from '../core/replica-in-mem-storage'
import createPubSub from '../../utils/pubsub'
import LockManager from './lock-manager'
import SyncAgent from './sync-agent'

export default class IsodbClient {
  events = createPubSub()

  _db: IsodbReplica
  _lockManager: LockManager
  _syncAgent: SyncAgent

  constructor() {
    this._db = new IsodbReplica(new ReplicaInMemStorage())
    this._db.storage.onUpdate(this._onUpdate)

    this._lockManager = new LockManager()

    this._syncAgent = new SyncAgent(this._db, this._lockManager)
    this._syncAgent.start()
  }

  _onUpdate = () => {
    this.events.emit('update')
  }

  canLockRecord(id: string) {
    return this._lockManager.canLockRecord(id)
  }
  lockRecord(id: string) {
    return this._lockManager.lockRecord(id)
  }

  destroy() {
    this._db.storage.offUpdate(this._onUpdate)
    this._syncAgent.stop()
  }
}
