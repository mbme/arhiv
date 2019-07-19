import fs from 'fs'
import { createLogger } from '~/logger'
import { IDict } from '~/utils'
import { Queue } from '~/utils/queue'
import { getMimeType } from '~/file-prober'
import {
  Server,
  MultipartBody,
  StringBody,
} from '~/http-server'
import { IChangeset } from '../types'
import { PrimaryDB } from '../primary'
import {
  isValidAuth,
  extractTokenCookie,
  resolveAsset,
  createToken,
} from './utils'

const log = createLogger('isodb-server')

export default function createServer(db: PrimaryDB, password = '', staticDirs: string[]) {
  const queue = new Queue()
  const server = new Server()

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

  server.post('/api/auth', async ({ req, res }) => {
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

  server.post('/api/changeset', async ({ res, req }) => {
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

    const changeset = JSON.parse(changesetField.value) as IChangeset

    const assets: IDict = {}
    for (const file of body.files) {
      assets[file.field] = file.file
    }

    res.body = await queue.push(() => db.applyChangeset(changeset, assets))
  })

  server.get('/api/file', async ({ req, res }) => {
    const fileId = req.url.query.fileId as string
    if (!fileId) {
      res.statusCode = 400

      return
    }

    const filePath = await queue.push(() => db.getAttachmentPath(fileId))

    if (filePath) {
      res.headers['Content-Disposition'] = `inline; filename=${fileId}`
      res.headers['Content-Type'] = await getMimeType(filePath)
      res.headers['Cache-Control'] = 'immutable, private, max-age=31536000' // max caching
      res.body = fs.createReadStream(filePath)
    } else {
      res.statusCode = 404
    }
  })

  // Handle assets + html5 history fallback
  server.get(() => true, async ({ req, res }) => {
    if (req.url.pathname!.startsWith('/api')) {
      res.statusCode = 404

      return
    }

    const fileName = req.url.path!.substring(1) // skip leading /
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
