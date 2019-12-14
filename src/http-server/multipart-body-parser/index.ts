import http from 'http'
import { Deferred } from '~/utils'

import { MultipartBody } from '../types'

import { ParserState } from './parser-state'

import { ParserIntro } from './parser-intro'
import { ParserHeaders } from './parser-headers'
import { ParserFieldBody } from './parser-field-body'
import { ParserFileBody } from './parser-file-body'

type Parser = ParserIntro
  | ParserHeaders
  | ParserFieldBody
  | ParserFileBody

export function parseMultipartBody(
  tmpDir: string,
  req: http.IncomingMessage,
  boundary: string,
): Promise<MultipartBody> {
  const deferred = new Deferred<MultipartBody>()
  const body = new MultipartBody()

  const state = new ParserState(boundary, tmpDir)
  let parser: Parser = new ParserIntro(state)

  function handleChunk(nextChunk: Buffer) {
    // FIXME make sure chunk.byteLength is bigger than boundary.byteLength
    // FIXME what if newChunk contains ALL THE DATA, i.e. headers and body
    // FIXME when to stop
    const complete = parser.processChunk(nextChunk)

    if (!complete) {
      return
    }

    if (parser instanceof ParserIntro) {
      parser = new ParserHeaders(state)

      return
    }

    if (parser instanceof ParserHeaders) {
      const {
        fieldName,
        isFile,
      } = parser.parseHeaders()

      parser = isFile
        ? new ParserFileBody(state, fieldName)
        : new ParserFieldBody(state, fieldName)

      return
    }

    if (parser instanceof ParserFieldBody) {
      body.fields.push({
        field: parser.name,
        value: parser.getValue(),
      })

      parser = new ParserHeaders(state)

      return
    }

    if (parser instanceof ParserFileBody) {
      body.files.push({
        field: parser.name,
        file: parser.file,
      })

      parser = new ParserHeaders(state)

      return
    }

    throw new Error('unreachable: got unexpected parser')
  }

  req.on('data', (nextChunk: Buffer) => {
    handleChunk(nextChunk)
  })

  req.on('error', (e) => {
    deferred.reject(e)
  })

  req.on('end', () => {
    // FIXME process remaining data & check if there was no redundant data
    deferred.resolve(body)
  })

  return deferred.promise
}
