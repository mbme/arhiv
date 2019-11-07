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
type UpgradeDB<S extends object> = (oldVersion: number, db: PIDB<S>) => void

export class PIDB<S extends object> {
  private constructor(
    private _db: IDBDatabase,
  ) { }

  static async open<S extends object>(name: string, version: number, upgrade: UpgradeDB<S>): Promise<PIDB<S>> {
    if (!Number.isInteger(version)) {
      throw new Error('version must be an integer')
    }

    const request = indexedDB.open(name, version)
    request.onupgradeneeded = (e: IDBVersionChangeEvent) => {
      upgrade(e.oldVersion, new PIDB(request.result))
    }

    const db = await request2promise(request)

    return new PIDB(db)
  }

  createObjectStore<StoreName extends keyof S, StoreTypeProps extends keyof S[StoreName]>(store: StoreName, keyPath: StoreTypeProps) {
    this._db.createObjectStore(store, { keyPath })
  }

  transaction<StoreName extends keyof S>(...stores: StoreName[]) {
    return new PIDBTransaction<S, StoreName>(this._db.transaction(stores, 'readonly'))
  }

  transactionRW<StoreName extends keyof S>(...stores: StoreName[]) {
    return new PIDBTransaction<S, StoreName>(this._db.transaction(stores, 'readwrite'))
  }

  getAll<StoreName extends keyof S>(store: StoreName) {
    return this.transaction(store).store(store).getAll()
  }

  get<StoreName extends keyof S>(store: StoreName, key: string) {
    return this.transaction(store).store(store).get(key)
  }

  put<StoreName extends keyof S>(store: StoreName, value: S[StoreName]) {
    return this.transactionRW(store).store(store).put(value)
  }
}

class PIDBTransaction<S extends object, StoreName extends keyof S> {
  constructor(
    private _tx: IDBTransaction,
  ) { }

  store<T extends StoreName, TType = S[T]>(name: T) {
    return new PIDBStore<TType>(this._tx.objectStore(name))
  }
}

class PIDBStore<T> {
  constructor(
    private _store: IDBObjectStore,
  ) { }

  getAll(): Promise<T[]> {
    return request2promise(this._store.getAll())
  }

  get(key: string): Promise<T | undefined> {
    return request2promise(this._store.get(key))
  }

  async put(value: T): Promise<void> {
    await request2promise(this._store.put(value))
  }
}
