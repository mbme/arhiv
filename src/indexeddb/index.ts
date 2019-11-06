function request2promise<R>(request: IDBRequest<R>): Promise<R> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result)
    request.onerror = reject
  })
}

// type DB<StoreNames extends string, StoredObjectType> = {
//   [StoreNames]: StoredObjectType,
// }

// type UpgradeDB = (oldVersion: number, db: IDBDatabase) => void
type UpgradeDB<S extends object> = (oldVersion: number, db: IDB<S>) => void

export class IDB<S extends object> {
  private constructor(
    private _db: IDBDatabase,
  ) { }

  static async open<S extends object>(name: string, version: number, upgrade: UpgradeDB<S>): Promise<IDB<S>> {
    if (!Number.isInteger(version)) {
      throw new Error('version must be an integer')
    }

    const request = indexedDB.open(name, version)
    request.onupgradeneeded = (e: IDBVersionChangeEvent) => {
      upgrade(e.oldVersion, new IDB(request.result))
    }

    const db = await request2promise(request)

    return new IDB(db)
  }

  createObjectStore<StoreName extends keyof S, StoreTypeProps extends keyof S[StoreName]>(store: StoreName, keyPath: StoreTypeProps) {
    this._db.createObjectStore(store, { keyPath })
  }

  getAll<StoreName extends keyof S, StoreType = S[StoreName]>(store: StoreName): Promise<StoreType[]> {
    return request2promise(this._db.transaction(store).objectStore(store).getAll())
  }
}
