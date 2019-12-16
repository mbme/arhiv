import { ParserState } from './parser-state'

export class ParserIntro {
  constructor(
    private _state: ParserState,
  ) {
  }

  processChunk(): boolean {
    // drop all teh data before the first boundary and the boundary itself
    const [foundBoundary] = this._state.consumeTillBoundary()

    return foundBoundary
  }
}
