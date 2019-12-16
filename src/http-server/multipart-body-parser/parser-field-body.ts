import { ParserState } from './parser-state'
import { stringifyChunks } from './utils'

export class ParserFieldBody {
  private _chunks: Buffer[] = []

  constructor(
    private _state: ParserState,
    public readonly name: string,
  ) {
  }

  processChunk(): boolean {
    const [foundBoundary, data] = this._state.consumeTillBoundary()

    this._chunks.push(data)

    return foundBoundary
  }

  getValue() {
    return stringifyChunks(this._chunks)
  }
}
