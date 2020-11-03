import fs from 'fs'
import { ParserState } from './parser-state'

export class ParserFileBody {
  public readonly file: string

  constructor(
    private _state: ParserState,
    public readonly name: string,
  ) {
    this.file = _state.genTempFile()
  }

  processChunk(): boolean {
    const [foundBoundary, data] = this._state.consumeTillBoundary()

    fs.appendFileSync(this.file, data)

    return foundBoundary
  }
}
