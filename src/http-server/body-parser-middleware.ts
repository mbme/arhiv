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
      const boundary = contentType.match('boundary=(.*)')?.[1]
      if (!boundary) {
        throw new Error(`multipart: boundary is missing: "${contentType}"`)
      }
      // TODO assert encoding

      let body: MultipartBody | undefined
      try {
        body = await parseMultipartBody(tmpDir, httpReq, boundary)

        req.body = body

        await next()
      } finally {
        await Promise.all(body?.files.map(item => removeFile(item.file)) || [])
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
}
