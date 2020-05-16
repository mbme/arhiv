import http from 'http'
import { createLogger } from '@v/logger'
import { MultipartBody } from '../types'
import { MultipartParser } from './parser'
import { extractBoundary } from './utils'

const log = createLogger('multipart-parser')

export function parseMultipartBody(req: http.IncomingMessage, tmpDir: string): Promise<MultipartBody> {
  return new Promise((resolve, reject) => {
    const contentType = req.headers['content-type'] || ''
    const boundary = extractBoundary(contentType)
    if (!boundary) {
      throw new Error(`multipart: boundary is missing: "${contentType}"`)
    }

    const parser = new MultipartParser(boundary, tmpDir)

    req.on('data', (chunk: Buffer) => {
      if (parser.isComplete()) {
        log.warn(`got data ${chunk.byteLength} after final boundary, ignoring`)

        return
      }

      parser.processChunk(chunk)
    })

    req.on('error', reject)

    req.on('end', () => {
      if (parser.isComplete()) {
        resolve(parser.getResult())
      } else {
        reject(new Error("multipart request didn't contain final boundary"))
      }
    })
  })
}
