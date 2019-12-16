import http from 'http'
import { createLogger } from '~/logger'
import { Deferred } from '~/utils'
import { MultipartBody } from '../types'
import { MultipartParser } from './parser'

const log = createLogger('multipart-parser')

export function parseMultipartBody(
  tmpDir: string,
  req: http.IncomingMessage,
  boundary: string,
): Promise<MultipartBody> {
  const deferred = new Deferred<MultipartBody>()
  const p = new MultipartParser(boundary, tmpDir)

  req.on('data', (chunk: Buffer) => {
    if (p.isComplete()) {
      log.warn(`got data ${chunk.byteLength} after final boundary, ignoring`)
      return
    }

    p.processChunk(chunk)
  })

  req.on('error', (e) => {
    deferred.reject(e)
  })

  req.on('end', () => {
    if (p.isComplete()) {
      deferred.resolve(p.getResult())
    } else {
      deferred.reject(new Error("multipart request didn't contain final boundary"))
    }
  })

  return deferred.promise
}
