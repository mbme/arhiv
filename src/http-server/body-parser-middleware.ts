import path from 'path'
import fs from 'fs'
import http from 'http'
// tslint:disable-next-line:match-default-export-name
import Busboy from 'busboy'
import {
  rmrfSync,
  createTempDir,
} from '../utils/fs'
import {
  ILazy,
  lazy,
} from '../utils/lazy'
import {
  readStreamAsString,
} from '../utils/node'
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

export async function bodyParserMiddleware({ req, httpReq }: IContext, next: Next) {
  if (!['POST', 'PUT'].includes(req.method)) {
    return next()
  }

  const contentType = req.headers['content-type'] || ''
  if (contentType.startsWith('multipart/form-data')) {
    const tmpDir = lazy(createTempDir)
    try {
      req.body = await readFormData(tmpDir, httpReq)

      await next()
    } finally {
      if (tmpDir.initialized) rmrfSync(await tmpDir.value)
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
