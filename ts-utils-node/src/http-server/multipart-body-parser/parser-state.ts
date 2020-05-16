import path from 'path'
import { Counter } from '@v/utils'
import {
  isSubarrayAt,
  CRLF,
} from './utils'

const newline = Buffer.from(CRLF, 'utf-8')
const closingDashes = Buffer.from('--', 'utf-8')

export class ParserState {
  public readonly boundary: Buffer

  // use newline as initial data cause first boundary may go without first newline
  // so now we can always match boundary as \r\n--boundary\r\n
  public chunk = newline

  public complete = false

  private _counter = new Counter()
  private _safeMargin: number

  constructor(
    boundary: string,
    private _tmpDir: string,
  ) {
    this.boundary = Buffer.from(`${CRLF}--${boundary}`, 'utf-8')
    this._safeMargin = this.boundary.byteLength + newline.byteLength
  }

  hasEnoughData() {
    return this.chunk.byteLength >= this._safeMargin
  }

  addChunk(newChunk: Buffer) {
    this.chunk = Buffer.concat([this.chunk, newChunk])
  }

  genTempFile() {
    return path.join(this._tmpDir, this._counter.incAndGet().toString())
  }

  private _detectBoundary(chunk: Buffer): [number, boolean] {
    const pos = chunk.indexOf(this.boundary)

    if (pos === -1) {
      return [-1, false]
    }

    const posAfterBoundary = pos + this.boundary.byteLength

    if (isSubarrayAt(chunk, posAfterBoundary, newline)) {
      return [pos, false]
    }

    if (isSubarrayAt(chunk, posAfterBoundary, closingDashes)) {
      return [pos, true]
    }

    return [-1, false]
  }

  consumeTillBoundary(): [boolean, Buffer] {
    const [pos, isFinal] = this._detectBoundary(this.chunk)

    if (pos === -1) {
      const safeMarginPos = this.chunk.byteLength - this._safeMargin
      const chunkToProceed = this.chunk.subarray(0, safeMarginPos)
      this.chunk = this.chunk.subarray(safeMarginPos)

      return [false, chunkToProceed]
    }

    if (isFinal) {
      this.complete = true
    }

    const chunkToProceed = this.chunk.subarray(0, pos)
    const boundaryLength = this.boundary.byteLength + (isFinal ? closingDashes.byteLength : newline.byteLength)
    this.chunk = this.chunk.subarray(pos + boundaryLength)

    return [true, chunkToProceed]
  }

  consumeTill(boundary: Buffer): [boolean, Buffer] {
    const pos = this.chunk.indexOf(boundary)

    if (pos === -1) {
      const safeMarginPos = this.chunk.byteLength - boundary.byteLength
      const chunkToProceed = this.chunk.subarray(0, safeMarginPos)
      this.chunk = this.chunk.subarray(safeMarginPos)

      return [false, chunkToProceed]
    }

    const chunkToProceed = this.chunk.subarray(0, pos)
    this.chunk = this.chunk.subarray(pos + boundary.byteLength)

    return [true, chunkToProceed]
  }
}
