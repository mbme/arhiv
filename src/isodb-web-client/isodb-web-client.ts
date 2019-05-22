import {
  IsodbReplica,
  ReplicaInMemStorage,
} from './replica'
import { createEventsPubSub } from './events'
import {
  LockAgent,
  SyncAgent,
  NetworkAgent,
  AuthAgent,
} from './agents'
import {
  NotesRepository,
  TracksRepository,
} from './records'

export class IsodbWebClient {
  public events = createEventsPubSub()

  private _db = new IsodbReplica(new ReplicaInMemStorage(), this.events)
  private _networkAgent = new NetworkAgent(this.events)
  private _lockAgent = new LockAgent(this.events)
  private _authAgent = new AuthAgent(this.events, this._networkAgent)
  private _syncAgent = new SyncAgent(this._db, this._lockAgent, this._networkAgent, this._authAgent)

  public notes = new NotesRepository(this._db, this._lockAgent)
  public tracks = new TracksRepository(this._db, this._lockAgent)

  start() {
    this._networkAgent.start()
    this._authAgent.start()
    this._syncAgent.start()
  }

  stop() {
    this._networkAgent.stop()
    this._authAgent.stop()
    this._syncAgent.stop()
  }

  isAuthorized() {
    return this._authAgent.isAuthorized()
  }

  async authorize(password: string) {
    return this._authAgent.authorize(password)
  }

  deauthorize() {
    this._authAgent.deauthorize()
  }

  syncNow() {
    this._syncAgent.syncNow()
  }
}
