import { ITrack } from '~/isodb-core/types'
import { BaseRepository } from './base-repository'
import { Track } from './track'

export class TracksRepository extends BaseRepository {
  private _intoTrack = (track: ITrack) => new Track(this._replica, this._lockAgent, track)

  createTrack() {
    const id = this.getRandomId()

    return this._intoTrack(Track.create(id))
  }

  getTracks(): Track[] {
    return this._replica.getRecords()
      .filter(Track.is)
      .map(this._intoTrack)
  }

  getTrack(id: string): Track | undefined {
    const record = this._replica.getRecord(id)
    if (Track.is(record)) {
      return this._intoTrack(record)
    }

    return undefined
  }
}
