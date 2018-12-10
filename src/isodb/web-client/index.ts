import IsodbReplica from '../core/replica';
import ReplicaInMemStorage from '../core/replica-in-mem-storage';
import createPubSub from '../../utils/pubsub';
import LockManager from './lock-manager';
import SyncManager from './sync-manager';
import { IReplicaStorage } from '../core/types';

export default class IsodbClient {
  events = createPubSub();

  _storage: IReplicaStorage
  _db: IsodbReplica
  _lockManager: LockManager
  _syncManager: SyncManager

  constructor() {
    this._storage = new ReplicaInMemStorage();
    this._storage.events.on('update', this._onUpdate);

    this._db = new IsodbReplica(this._storage);
    this._lockManager = new LockManager();

    this._syncManager = new SyncManager(this._db, this._lockManager);
    this._syncManager.start();
  }

  _onUpdate = () => {
    this.events.emit('update');
  };

  canLockRecord(id: string) {
    return this._lockManager.canLockRecord(id)
  }
  lockRecord(id: string) {
    return this._lockManager.lockRecord(id);
  }

  destroy() {
    this._storage.events.off('update', this._onUpdate);
    this._syncManager.stop();
  }
}
