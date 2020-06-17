import { Obj } from '@v/utils'

function request2promise<R>(request: IDBRequest<R>): Promise<R> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result)
    request.onerror = reject
  })
}

export async function applyForPersistentStorage(): Promise<boolean> {
  if (navigator.storage?.persist) {
    return navigator.storage.persist()
  }

  return false
}

// type DB<StoreNames extends string, StoredObjectType> = {
//   [StoreNames]: StoredObjectType,
// }

export class TIDB<ObjectStores extends Obj> {
  private constructor(
    private _db: IDBDatabase,
    public readonly version: number,
  ) { }

  static async open<ObjectStores extends Obj>(
    name: string,
    version: number,
    upgrade: (oldVersion: number, db: TIDB<ObjectStores>) => void,
  ): Promise<TIDB<ObjectStores>> {
    if (!Number.isInteger(version)) {
      throw new Error('version must be an integer')
    }

    const request = indexedDB.open(name, version)

    request.onupgradeneeded = (e: IDBVersionChangeEvent) => {
      upgrade(e.oldVersion, new TIDB(request.result, version))
    }

    request.onblocked = () => {
      throw new Error('We need to update the db schema. Please close all other tabs with this db!')
    }

    const db = await request2promise(request)

    return new TIDB(db, version)
  }

  createObjectStore<StoreName extends keyof ObjectStores>(
    store: StoreName,
    keyPath: keyof ObjectStores[StoreName],
  ) {
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

  async put<StoreName extends keyof ObjectStores>(
    store: StoreName,
    value: ObjectStores[StoreName],
  ) {
    await this.transactionRW(store).store(store).put(value)
  }

  async putAll<StoreName extends keyof ObjectStores>(
    store: StoreName,
    values: ObjectStores[StoreName][],
  ) {
    await this.transactionRW(store).store(store).putAll(values)
  }

  async delete<StoreName extends keyof ObjectStores>(store: StoreName, key: string) {
    await this.transactionRW(store).store(store).delete(key)
  }
}

export class TIDBTransaction<ObjectStores extends Obj, StoreName extends keyof ObjectStores> {
  constructor(private _tx: IDBTransaction) { }

  store<T extends StoreName, TType = ObjectStores[T]>(name: T) {
    return new TIDBStore<TType>(this._tx.objectStore(name))
  }
}

class TIDBStore<T> {
  constructor(private _store: IDBObjectStore) { }

  getAll(): Promise<T[]> {
    return request2promise<T[]>(this._store.getAll())
  }

  async getAllKeys(): Promise<string[]> {
    const keys = await request2promise(this._store.getAllKeys())

    return keys.map(item => item.toString())
  }

  get(key: string): Promise<T | undefined> {
    return request2promise<T>(this._store.get(key))
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

  async clear(): Promise<void> {
    await request2promise(this._store.clear())
  }
}
