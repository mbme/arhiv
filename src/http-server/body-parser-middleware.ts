import path from 'path'
import fs from 'fs'
import http from 'http'
import Busboy from 'busboy'
import { rmrfSync, createTempDir } from '../fs/utils'
import { ILazy, lazy } from '../utils'
import { IContext, Next, HttpMethod } from './index'
import { readStreamAsString } from '../utils/node'
import { IMultipartField, IMultipartFile, MultipartBody, JSONBody, StringBody } from './types'

// Extract action & assets from multipart/form-data POST request
function readFormData(tmpDir: ILazy<Promise<string>>, req: http.IncomingMessage): Promise<MultipartBody> {
  const fields: IMultipartField[] = []
  const files: IMultipartFile[] = []
  let fileCounter = 0

  const busboy = new Busboy({ headers: req.headers })

  busboy.on('field', (field, value) => {
    fields.push({ field, value })
  })

  busboy.on('file', async (field, fileStream) => {
    const file = path.join(await tmpDir.value, (fileCounter += 1).toString())
    fileStream.pipe(fs.createWriteStream(file))
    files.push({ field, file })
  })

  return new Promise((resolve) => {
    busboy.on('finish', () => resolve(new MultipartBody(fields, files)))
    req.pipe(busboy)
  })
}

export default async function bodyParserMiddleware({ req, httpReq }: IContext, next: Next) {
  if (![HttpMethod.POST, HttpMethod.PUT].includes(req.method)) return next()

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
    req.body = new JSONBody(JSON.parse(await readStreamAsString(httpReq)))
    return next()
  }

  // just string
  req.body = new StringBody(await readStreamAsString(httpReq))
  return next()
}
