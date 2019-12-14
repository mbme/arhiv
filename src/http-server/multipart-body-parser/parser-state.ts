import path from 'path'
import { Counter } from '~/utils'

export class ParserState {
  public readonly boundary: Buffer

  public prevChunk = Buffer.alloc(0)

  private _counter = new Counter()

  constructor(
    boundary: string,
    private _tmpDir: string,
  ) {
    this.boundary = Buffer.from(`--${boundary}`, 'utf-8')
  }

  genTempFile() {
    return path.join(this._tmpDir, this._counter.incAndGet.toString())
  }
}
