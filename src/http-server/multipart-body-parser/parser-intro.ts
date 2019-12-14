import { ParserState } from './parser-state'

export class ParserIntro {
  constructor(
    private _state: ParserState,
  ) {
  }

  processChunk(nextChunk: Buffer): boolean {
    const chunk = Buffer.concat([this._state.prevChunk, nextChunk])

    const pos = chunk.indexOf(this._state.boundary)

    if (pos === -1) {
      this._state.prevChunk = nextChunk

      return false
    }

    // drop all teh data before the first boundary and the boundary itself
    this._state.prevChunk = chunk.subarray(pos + this._state.boundary.byteLength + 1)

    return true
  }
}
