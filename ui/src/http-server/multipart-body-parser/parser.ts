import { MultipartBody } from '../types'

import { ParserState } from './parser-state'

import { ParserIntro } from './parser-intro'
import { ParserHeaders } from './parser-headers'
import { ParserFieldBody } from './parser-field-body'
import { ParserFileBody } from './parser-file-body'

type Parser = ParserIntro | ParserHeaders | ParserFieldBody | ParserFileBody

export class MultipartParser {
  private _body = new MultipartBody()
  private _state: ParserState
  private _parser: Parser

  constructor(boundary: string, tmpDir: string) {
    this._state = new ParserState(boundary, tmpDir)
    this._parser = new ParserIntro(this._state)
  }

  isComplete(): boolean {
    return this._state.complete
  }

  processChunk(chunk: Buffer) {
    this._state.addChunk(chunk)

    while (this._state.hasEnoughData()) {
      const complete = this._parser.processChunk()

      if (!complete) { // wait for more chunks, can't proceed further
        return
      }

      this._switchParser()
    }
  }

  getResult(): MultipartBody {
    return this._body
  }

  private _switchParser() {
    if (this._parser instanceof ParserIntro) {
      this._parser = new ParserHeaders(this._state)

      return
    }

    if (this._parser instanceof ParserHeaders) {
      const {
        fieldName,
        isFile,
      } = this._parser.parseHeaders()

      this._parser = isFile
        ? new ParserFileBody(this._state, fieldName)
        : new ParserFieldBody(this._state, fieldName)

      return
    }

    if (this._parser instanceof ParserFieldBody) {
      this._body.fields.push({
        field: this._parser.name,
        value: this._parser.getValue(),
      })

      this._parser = new ParserHeaders(this._state)

      return
    }

    if (this._parser instanceof ParserFileBody) {
      this._body.files.push({
        field: this._parser.name,
        file: this._parser.file,
      })

      this._parser = new ParserHeaders(this._state)

      return
    }

    throw new Error('unreachable: got unexpected parser')
  }
}
