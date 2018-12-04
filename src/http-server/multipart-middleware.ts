import path from 'path'
import fs from 'fs'
import http from 'http'
import Busboy from 'busboy'
import { rmrfSync, createTempDir } from '../fs/utils';
import { ILazy, lazy } from '../utils'
import { IContext, Next, HttpMethod } from './index'

interface IMultipartField {
  field: string
  value: string
}
interface IMultipartFile {
  field: string
  file: string
}

interface IMultipartBody {
  fields: IMultipartField[]
  files: IMultipartFile[]
}

// Extract action & assets from multipart/form-data POST request
function readFormData(tmpDir: ILazy<Promise<string>>, req: http.IncomingMessage): Promise<IMultipartBody> {
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
    busboy.on('finish', () => resolve({ fields, files }))
    req.pipe(busboy)
  })
}

export async function multipartMiddleware({ req, httpReq }: IContext, next: Next) {
  if (req.method !== HttpMethod.POST) return next()

  const isMultipartRequest = (req.headers['content-type'] || '').startsWith('multipart/form-data');
  if (!isMultipartRequest) return next()

  const tmpDir = lazy(createTempDir)
  try {
    req.body = await readFormData(tmpDir, httpReq)

    await next()
  } finally {
    if (tmpDir.initialized) rmrfSync(await tmpDir.value);
  }

}
