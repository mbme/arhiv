import http from 'http'
import { createLogger } from '~/logger'
import { Deferred } from '~/utils'
import { MultipartBody } from '../types'
import { MultipartParser } from './parser'
import { extractBoundary } from './utils'

const log = createLogger('multipart-parser')

export function parseMultipartBody(req: http.IncomingMessage, tmpDir: string): Promise<MultipartBody> {
  const contentType = req.headers['content-type'] || ''
  const boundary = extractBoundary(contentType)
  if (!boundary) {
    throw new Error(`multipart: boundary is missing: "${contentType}"`)
  }
  // TODO assert encoding

  const deferred = new Deferred<MultipartBody>()
  const parser = new MultipartParser(boundary, tmpDir)

  req.on('data', (chunk: Buffer) => {
    if (parser.isComplete()) {
      log.warn(`got data ${chunk.byteLength} after final boundary, ignoring`)
      return
    }

    parser.processChunk(chunk)
  })

  req.on('error', (e) => {
    deferred.reject(e)
  })

  req.on('end', () => {
    if (parser.isComplete()) {
      deferred.resolve(parser.getResult())
    } else {
      deferred.reject(new Error("multipart request didn't contain final boundary"))
    }
  })

  return deferred.promise
}
