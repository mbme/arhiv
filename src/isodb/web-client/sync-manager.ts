import LockManager from './lock-manager'
import { IPatchResponse, ChangedRecord, Record } from '../core/types'
import ReplicaDB from '../core/replica'

async function pushChanges(
  rev: number,
  records: ChangedRecord[],
  attachments: { [hash: string]: Blob }
): Promise<IPatchResponse> {
  const data = new FormData()
  data.append('rev', rev.toString())
  data.append('records', JSON.stringify(records))
  for (const [hash, blob] of Object.entries(attachments)) {
    data.append(hash, blob)
  }

  const response = await fetch('/api/changes', {
    method: 'post',
    credentials: 'include',
    body: data,
  })

  if (!response.ok) {
    throw new Error(`Server responded with code ${response.status}`)
  }

  return response.json()
}

// TODO logs
// TODO listen to network
// TODO circuit breaker
export default class SyncManager {
  _syncIntervalId: number | undefined

  constructor(public _replica: ReplicaDB, public _lockManager: LockManager) { }

  _merge = async (_base: Record, _newBase: Record, local: ChangedRecord) => {
    // FIXME implement merge
    return local
  }

  async _sync() {
    if (!this._lockManager.lockDB()) return false

    try {
      let tries = 0
      while (true) {
        const result = await pushChanges(
          this._replica.getRev(),
          this._replica.storage.getLocalRecords(),
          this._replica.storage.getLocalAttachments()
        )

        await this._replica.applyPatch(result, this._merge)

        if (result.applied) {
          return true
        }

        tries += 1
        if (tries > 2) {
          throw new Error('Failed to sync, try again later')
        }
      }
    } finally {
      this._lockManager.unlockDB()
    }
  }

  start() {
    // tslint:disable-next-line:no-floating-promises
    this._sync()

    // schedule sync once per minute
    this._syncIntervalId = window.setInterval(this._sync, 60 * 1000)
  }

  stop() {
    clearInterval(this._syncIntervalId)
  }
}
