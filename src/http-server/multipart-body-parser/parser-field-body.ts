import { ParserState } from './parser-state'
import { stringifyChunks } from './utils'

export class ParserFieldBody {
  private _chunks: Buffer[] = []

  constructor(
    private _state: ParserState,
    public readonly name: string,
  ) {
  }

  processChunk(nextChunk: Buffer): boolean {
    const chunk = Buffer.concat([this._state.prevChunk, nextChunk])

    const pos = chunk.indexOf(this._state.boundary)

    if (pos === -1) {
      this._chunks.push(this._state.prevChunk)
      this._state.prevChunk = nextChunk

      return false
    }

    this._chunks.push(chunk.subarray(0, pos))
    this._state.prevChunk = chunk.subarray(pos + this._state.boundary.byteLength + 1)

    return true
  }

  getValue() {
    return stringifyChunks(this._chunks)
  }
}
