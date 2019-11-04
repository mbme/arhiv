export function request2promise<R>(request: IDBRequest<R>): Promise<R> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result)
    request.onerror = reject
  })
}

type UpgradeDB = (oldVersion: number, db: IDBDatabase) => void
export function openDB(name: string, version: number, upgrade: UpgradeDB) {
  if (!Number.isInteger(version)) {
    throw new Error('version must be an integer')
  }

  const request = indexedDB.open(name, version)
  request.onupgradeneeded = (e: IDBVersionChangeEvent) => {
    upgrade(e.oldVersion, request.result)
  }

  return request2promise(request)
}

export class IDB {

}
