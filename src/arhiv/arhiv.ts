import {
  ReplicaInMemStorage,
  ReplicaManager,
} from '~/isodb/replica'
import { NetworkManager } from './network-manager'
import { ArhivReplica } from './types'
import {
  NotesRepository,
  TracksRepository,
  AttachmentsRepository,
} from './repositories'

export class Arhiv {
  net = new NetworkManager()
  private _replica: ArhivReplica = new ReplicaManager(new ReplicaInMemStorage())
  private _syncIntervalId: number | undefined

  attachments = new AttachmentsRepository(this._replica)
  notes = new NotesRepository(this._replica)
  tracks = new TracksRepository(this._replica)

  constructor() {
    this.net.$authorized.subscribe((isAuthorized) => {
      if (isAuthorized) {
        this.syncNow()
      }
    })

    this._replica.$syncState.subscribe((syncState) => {
      if (syncState === 'merge-conflicts-resolved') {
        this.syncNow()
      }
    })
  }

  private _sync = () => {
    // tslint:disable-next-line:no-floating-promises
    this._replica.sync(this.net.syncChanges)
  }

  syncNow = () => {
    this._sync()

    // schedule sync once per minute
    clearInterval(this._syncIntervalId)
    this._syncIntervalId = window.setInterval(this._sync, 60 * 1000)
  }

  stop() {
    clearInterval(this._syncIntervalId)

    this._replica.stop()
    this.net.stop()
  }
}
