import fs from 'fs'
import { createLogger } from '~/logger'
import {
  Dict,
} from '~/utils'
import { Queue } from '~/utils/queue'
import { getMimeType } from '~/file-prober'
import {
  HTTPServer,
  MultipartBody,
  StringBody,
} from '~/http-server'
import {
  IChangeset,
  IDocument,
} from '../types'
import { ArhivDB } from './db'
import {
  isValidAuth,
  extractTokenCookie,
  resolveAsset,
  createToken,
} from './utils'
import { PathMatcher } from '~/utils/path-matcher'

const log = createLogger('arhiv-server')

export default function createServer(db: ArhivDB<IDocument>, password = '', staticDirs: string[]) {
  const queue = new Queue()
  const server = new HTTPServer()

  log.debug('static dirs: ', staticDirs.join(', '))

  server.use(async function authMiddleware(context, next) {
    const { req, res } = context

    res.headers['Referrer-Policy'] = 'no-referrer'

    const isAuthorized = isValidAuth(extractTokenCookie(req.headers.cookie || ''), password)
    if (!isAuthorized
      && req.url.pathname!.startsWith('/api')
      && req.url.pathname !== '/api/auth') {
      res.statusCode = 403

      return
    }

    await next()
  })

  // POST /api/auth
  server.post(PathMatcher.create().string('api').string('auth'), async ({ req, res }) => {
    const body = req.body!
    if (!(body instanceof StringBody)) {
      res.statusCode = 415

      return
    }

    if (body.value !== password) {
      res.statusCode = 401

      return
    }

    res.headers['set-cookie'] = createToken(password)
  })

  // POST /api/changeset
  server.post(PathMatcher.create().string('api').string('changeset'), async ({ res, req }) => {
    const body = req.body!
    if (!(body instanceof MultipartBody)) {
      res.statusCode = 415

      return
    }

    const changesetField = body.getField('changeset')
    if (!changesetField) {
      log.error(`changeset field is mandatory`)
      res.statusCode = 400

      return
    }

    const changeset = JSON.parse(changesetField.value) as IChangeset<IDocument>

    const assets: Dict = {}
    for (const file of body.files) {
      assets[file.field] = file.file
    }

    res.body = await queue.push(() => db.applyChangeset(changeset, assets))
  })

  // GET /api/file/:fileId
  server.get(PathMatcher.create().string('api').string('file').param('fileId'), async ({ res }, { fileId }) => {
    const filePath = await queue.push(() => db.getAttachmentDataPath(fileId))

    if (filePath) {
      res.headers['Content-Disposition'] = `inline; filename=${fileId}`
      res.headers['Content-Type'] = await getMimeType(filePath)
      res.headers['Cache-Control'] = 'immutable, private, max-age=31536000' // max caching
      res.body = fs.createReadStream(filePath)
    } else {
      res.statusCode = 404
    }
  })

  // GET /api/*
  server.get(PathMatcher.create().string('api').everything(), ({ res }) => {
    res.statusCode = 400
  })

  // Handle assets + html5 history fallback
  // GET /*
  server.get(PathMatcher.create().everything(), async ({ res }, { everything }) => {
    const fileName = everything.join('/')
    const filePath = await resolveAsset(staticDirs, fileName || 'index.html')
      || await resolveAsset(staticDirs, 'index.html') // html5 history fallback

    if (filePath) {
      res.headers['Content-Type'] = await getMimeType(filePath)
      res.body = fs.createReadStream(filePath)
    } else {
      res.statusCode = 404
    }
  })

  return {
    start(port: number) {
      return server.start(port)
    },

    stop() {
      return Promise.all([server.stop(), queue.close()])
    },
  }
}
