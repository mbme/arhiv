import path from 'path'
import fs from 'fs'
import http from 'http'
// tslint:disable-next-line:match-default-export-name
import Busboy from 'busboy'
import {
  Counter,
} from '~/utils'
import {
  rmrfSync,
  createTempDir,
} from '~/utils/fs'
import {
  ILazy,
  lazy,
} from '~/utils/lazy'
import {
  readStreamAsString,
} from '~/utils/node'
import {
  MultipartBody,
  JSONBody,
  StringBody,
  IContext,
  Next,
} from './types'

// Extract action & assets from multipart/form-data POST request
function readFormData(tmpDir: ILazy<Promise<string>>, req: http.IncomingMessage): Promise<MultipartBody> {
  const body = new MultipartBody()
  let fileCounter = 0

  const busboy = new Busboy({ headers: req.headers })

  busboy.on('field', (field: string, value: string) => {
    body.fields.push({ field, value })
  })

  busboy.on('file', async (field, fileStream) => {
    const file = path.join(await tmpDir.value, (fileCounter += 1).toString())
    fileStream.pipe(fs.createWriteStream(file))
    body.files.push({ field, file })
  })

  return new Promise((resolve) => {
    busboy.on('finish', () => resolve(body))
    req.pipe(busboy)
  })
}

type MultipartParserState = {
  type: 'before'
} | {
  type: 'headers'
  content: Buffer[]
} | {
  type: 'body-field'
  name: string
  content: Buffer[]
} | {
  type: 'body-file',
  name: string
  file: string
  writeStream: fs.WriteStream,
}

function readFormData1(tmpDir: string, req: http.IncomingMessage, boundaryStr: string): Promise<MultipartBody> {
  const body = new MultipartBody()
  const counter = new Counter()
  const boundary = Buffer.from(`--${boundaryStr}\n`, 'utf-8')
  const lastBoundary = Buffer.from(`--${boundaryStr}--`, 'utf-8')
  const headersBoundary = Buffer.from('\n\n', 'utf-8')

  let state: MultipartParserState = {
    type: 'before'
  }

  let prevChunk = Buffer.alloc(0)

  req.on('data', (newChunk: Buffer) => {
    const chunk = Buffer.concat([prevChunk, newChunk])
    //  FIXME what if newChunk contains ALL THE DATA, i.e. headers and body
    switch (state.type) {
      case 'before': {
        const pos = chunk.indexOf(boundary)

        if (pos === -1) {
          prevChunk = newChunk;
        } else {
          // drop all teh data before the first boundary and the boundary itself
          prevChunk = chunk.subarray(pos + boundary.byteLength + 1)
          state = {
            type: 'headers',
            content: [],
          }
        }

        break;
      }

      case 'headers': {
        const pos = chunk.indexOf(headersBoundary)

        if (pos === -1) {
          state.content.push(prevChunk)
          prevChunk = newChunk;
        } else {
          state.content.push(chunk.subarray(0, pos))
          prevChunk = chunk.subarray(pos + headersBoundary.byteLength + 1)

          const headers = Buffer.concat(state.content).toString('utf-8').split('\n')
          const contentDispositionHeader = headers.find(header => header.toLowerCase().startsWith('content-disposition:'))
          if (!contentDispositionHeader) {
            // FIXME close streams property
            throw new Error('multipart body: Content-Disposition header is missing')
          }

          const fieldName = contentDispositionHeader.match(/name="(.*)"/) // FIXME check this regexp
          if (!fieldName) {
            throw new Error("multipart body: Content-Disposition header doesn't include name")
          }

          const isFile = contentDispositionHeader.includes('filename')
          if (isFile) {
            const file = path.join(tmpDir, counter.incAndGet.toString())
            state = {
              type: 'body-file',
              name: fieldName[1],
              file,
              writeStream: fs.createWriteStream(file)
            }
          } else {
            state = {
              type: 'body-field',
              name: fieldName[1],
              content: [],
            }
          }
        }

        break;
      }

      case 'body-field': {
        const pos = chunk.indexOf(boundary)
        if (pos === -1) {
          state.content.push(prevChunk)
          prevChunk = newChunk;
        } else {
          state.content.push(chunk.subarray(0, pos))
          prevChunk = chunk.subarray(pos + boundary.byteLength + 1)
          body.fields.push({ field: state.name, value: Buffer.concat(state.content).toString('utf-8') })
          state = {
            type: 'headers',
            content: [],
          }
        }

        break;
      }

      case 'body-file': {
        const pos = chunk.indexOf(boundary)
        if (pos === -1) {
          state.writeStream.write(prevChunk) // TODO check if written
          prevChunk = newChunk;
        } else {
          state.writeStream.write(chunk.subarray(0, pos)) // TODO check if written
          state.writeStream.end()
          prevChunk = chunk.subarray(pos + boundary.byteLength + 1)
          body.files.push({
            field: state.name,
            file: state.file,
          })
          state = {
            type: 'headers',
            content: [],
          }
        }

        break;
      }
    }
  })

  // find first --boundary in stream

  // read headers (Content-Disposition, Content-Type) till 2 newlines
  // parse headers
  // read body till newline --boundary

  // closing --boundary--

}

export async function bodyParserMiddleware({ req, httpReq }: IContext, next: Next) {
  if (!['POST', 'PUT'].includes(req.method)) {
    return next()
  }

  const contentType = req.headers['content-type'] || ''
  if (contentType.startsWith('multipart/form-data')) {
    const boundary = contentType.match('boundary=(.*)')?.[1]
    if (!boundary) {
      throw new Error(`multipart: boundary is missing: "${contentType}"`)
    }
    // TODO assert encoding

    const tmpDir = lazy(createTempDir)
    try {
      req.body = await readFormData1(tmpDir, httpReq, boundary)

      await next()
    } finally {
      if (tmpDir.initialized) {
        rmrfSync(await tmpDir.value)
      }
    }

    return
  }

  if (contentType.startsWith('application/json')) {
    req.body = new JSONBody(JSON.parse(await readStreamAsString(httpReq)) as object)

    return next()
  }

  // just string
  req.body = new StringBody(await readStreamAsString(httpReq))

  return next()
}
