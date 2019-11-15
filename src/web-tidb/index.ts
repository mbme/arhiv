function request2promise<R>(request: IDBRequest<R>): Promise<R> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result)
    request.onerror = reject
  })
}

// type DB<StoreNames extends string, StoredObjectType> = {
//   [StoreNames]: StoredObjectType,
// }

export class TIDB<ObjectStores extends object> {
  private constructor(
    private _db: IDBDatabase,
    public readonly oldVersion: number,
    public readonly version: number,
  ) { }

  static async open<S extends object>(name: string, version: number): Promise<TIDB<S>> {
    if (!Number.isInteger(version)) {
      throw new Error('version must be an integer')
    }

    let oldVersion = version
    const request = indexedDB.open(name, version)

    request.onupgradeneeded = (e: IDBVersionChangeEvent) => {
      oldVersion = e.oldVersion
    }

    request.onblocked = () => {
      throw new Error('We need to update the db schema. Please close all other tabs with this db!')
    }

    const db = await request2promise(request)

    return new TIDB(db, oldVersion, version)
  }

  isUpgradeNeeded() {
    return this.oldVersion !== this.version
  }

  createObjectStore<StoreName extends keyof ObjectStores>(store: StoreName, keyPath: keyof ObjectStores[StoreName]) {
    this._db.createObjectStore(store, { keyPath })
  }

  transaction<StoreName extends keyof ObjectStores>(...stores: StoreName[]) {
    return new TIDBTransaction<ObjectStores, StoreName>(this._db.transaction(stores, 'readonly'))
  }

  transactionRW<StoreName extends keyof ObjectStores>(...stores: StoreName[]) {
    return new TIDBTransaction<ObjectStores, StoreName>(this._db.transaction(stores, 'readwrite'))
  }

  getAll<StoreName extends keyof ObjectStores>(store: StoreName) {
    return this.transaction(store).store(store).getAll()
  }

  get<StoreName extends keyof ObjectStores>(store: StoreName, key: string) {
    return this.transaction(store).store(store).get(key)
  }

  put<StoreName extends keyof ObjectStores>(store: StoreName, value: ObjectStores[StoreName]) {
    return this.transactionRW(store).store(store).put(value)
  }

  delete<StoreName extends keyof ObjectStores>(store: StoreName, key: string) {
    return this.transactionRW(store).store(store).delete(key)
  }
}

class TIDBTransaction<S extends object, StoreName extends keyof S> {
  constructor(private _tx: IDBTransaction) { }

  store<T extends StoreName, TType = S[T]>(name: T) {
    return new TIDBStore<TType>(this._tx.objectStore(name))
  }
}

class TIDBStore<T> {
  constructor(private _store: IDBObjectStore) { }

  getAll(): Promise<T[]> {
    return request2promise(this._store.getAll())
  }

  get(key: string): Promise<T | undefined> {
    return request2promise(this._store.get(key))
  }

  async put(value: T): Promise<void> {
    await request2promise(this._store.put(value))
  }

  async putAll(values: readonly T[]): Promise<void> {
    await Promise.all(values.map(value => this.put(value)))
  }

  async delete(key: string): Promise<void> {
    await request2promise(this._store.delete(key))
  }
}
