import fs from 'fs'
import { ParserState } from './parser-state'

export class ParserFileBody {
  public readonly file: string

  private _ws: fs.WriteStream

  constructor(
    private _state: ParserState,
    public readonly name: string,
  ) {
    this.file = _state.genTempFile()
    this._ws = fs.createWriteStream(this.file)
  }

  processChunk(): boolean {
    const [foundBoundary, data] = this._state.consumeTillBoundary()

    this._ws.write(data)

    if (foundBoundary) {
      this._ws.end()
    }

    return foundBoundary
  }
}
