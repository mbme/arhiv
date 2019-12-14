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

  private _writeChunk(chunk: Buffer) {
    if (!this._ws.write(chunk)) {
      throw new Error('body-file parser failed to write chunk into the stream')
    }
  }

  processChunk(nextChunk: Buffer): boolean {
    const chunk = Buffer.concat([this._state.prevChunk, nextChunk])

    const pos = chunk.indexOf(this._state.boundary)

    if (pos === -1) {
      this._writeChunk(chunk)
      this._state.prevChunk = nextChunk

      return false
    }

    this._writeChunk(chunk.subarray(0, pos))
    this._ws.end()

    this._state.prevChunk = chunk.subarray(pos + this._state.boundary.byteLength + 1)

    return true
  }
}
