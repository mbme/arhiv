import {
  removeFile,
} from '~/utils/fs'
import {
  readStreamAsString,
} from '~/utils/node'
import {
  JSONBody,
  StringBody,
  IContext,
  Next,
  MultipartBody,
} from './types'
import { parseMultipartBody } from './multipart-body-parser'

export function createBodyParserMiddleware(tmpDir: string) {
  return async function bodyParserMiddleware({ req, httpReq }: IContext, next: Next) {
    if (!['POST', 'PUT'].includes(req.method)) {
      return next()
    }

    const contentType = req.headers['content-type'] || ''
    if (contentType.startsWith('multipart/form-data')) {
      let body: MultipartBody | undefined
      try {
        body = await parseMultipartBody(httpReq, tmpDir)

        req.body = body

        await next()
      } finally {
        await Promise.all(body?.files.map(item => removeFile(item.file, true)) || [])
      }

      return Promise.resolve()
    }

    if (contentType.startsWith('application/json')) {
      req.body = new JSONBody(JSON.parse(await readStreamAsString(httpReq)) as object)

      return next()
    }

    // just string
    req.body = new StringBody(await readStreamAsString(httpReq))

    return next()
  }
}
