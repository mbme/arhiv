import path from 'path'
import fs from 'fs'
import { createLogger } from '../../logger'
import createQueue from '../../utils/queue'
import { getMimeType } from '../../file-prober'
import { Server, MultipartBody } from '../../http-server'
import {
  isValidAuth,
  extractToken,
  resolveAsset,
} from './utils'
import PrimaryDB from '../core/primary'
import { IPatchResponse } from '../core/types'

const log = createLogger('isodb-server')

const STATIC_DIR = path.join(__dirname, '../../web-app/static')
const DIST_DIR = path.join(__dirname, '../../../dist')

export default function createServer(db: PrimaryDB, password = '') {
  const queue = createQueue()
  const server = new Server()

  server.use(async function bootstrapMiddleware(context, next) {
    const { req, res } = context

    res.headers['Referrer-Policy'] = 'no-referrer'

    const isAuthorized = isValidAuth(extractToken(req.headers.cookie || ''), password)
    if (req.url.pathname!.startsWith('/api') && !isAuthorized) {
      res.statusCode = 403
      return
    }

    await next()
  })

  server.post('/api/patch', async ({ res, req }) => {
    const body = req.body!
    if (!(body instanceof MultipartBody)) {
      res.statusCode = 415
      return
    }

    const revField = body.getField('rev')
    if (!revField) {
      log.error(`rev field is mandatory`)
      res.statusCode = 400
      return
    }

    const recordsField = body.getField('records')
    if (!recordsField) {
      log.error(`records field is mandatory`)
      res.statusCode = 400
      return
    }

    const rev = parseInt(revField.value, 10)
    const records = JSON.parse(recordsField.value)

    // TODO validate hashes
    const assets: { [hash: string]: string } = {}
    for (const file of body.files) {
      assets[file.field] = file.file
    }

    const success = await queue.push(() => db.applyChanges(rev, records, assets))
    const patch: IPatchResponse = {
      applied: success,
      baseRev: rev,
      currentRev: db.getRev(),
      records: db.getAll(rev),
    }
    res.body = patch
  })

  server.get('/api', async ({ req, res }) => {
    const fileId = req.url.query.fileId as string
    if (!fileId) {
      res.statusCode = 400
      return
    }

    const filePath = await queue.push(() => db.getAttachment(fileId))

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
    const fileName = req.url.path!.substring(1) // skip leading /
    const filePath = await resolveAsset(STATIC_DIR, fileName)
      || await resolveAsset(DIST_DIR, fileName)
      || await resolveAsset(STATIC_DIR, 'index.html') // html5 history fallback

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
