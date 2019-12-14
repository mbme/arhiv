import { ParserState } from './parser-state'
import { stringifyChunks } from './utils'

const headersBoundary = Buffer.from('\n\n', 'utf-8')

const nameRegexp = /name="([^""]*)"/

export class ParserHeaders {
  private _chunks: Buffer[] = []

  constructor(
    private _state: ParserState,
  ) {
  }

  processChunk(nextChunk: Buffer): boolean {
    const chunk = Buffer.concat([this._state.prevChunk, nextChunk])

    const pos = chunk.indexOf(headersBoundary)

    if (pos === -1) {
      this._chunks.push(this._state.prevChunk)
      this._state.prevChunk = nextChunk

      return false
    }

    this._chunks.push(chunk.subarray(0, pos))
    this._state.prevChunk = chunk.subarray(pos + headersBoundary.byteLength + 1)

    return true
  }

  parseHeaders() {
    const headers = stringifyChunks(this._chunks).split('\n')

    const contentDispositionHeader = headers.find(header => header.toLowerCase().startsWith('content-disposition:'))
    if (!contentDispositionHeader) {
      throw new Error('multipart body: Content-Disposition header is missing')
    }

    const fieldName = contentDispositionHeader.match(nameRegexp)?.[1]
    if (!fieldName) {
      throw new Error("multipart body: Content-Disposition header doesn't include name")
    }

    const isFile = contentDispositionHeader.includes('filename')

    return {
      fieldName,
      isFile,
    }
  }
}
