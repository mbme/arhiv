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

  processChunk(): boolean {
    const [foundBoundary, data] = this._state.consumeTill(headersBoundary)

    this._chunks.push(data)

    return foundBoundary
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
