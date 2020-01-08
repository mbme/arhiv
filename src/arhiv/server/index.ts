import fs from 'fs'
import {
  createLogger,
} from '~/logger'
import {
  Dict,
} from '~/utils'
import {
  Queue,
} from '~/utils/queue'
import {
  pathMatcher as pm,
} from '~/utils/path-matcher'
import {
  createTempDir,
  rmrfSync,
} from '~/utils/fs'
import {
  getMimeType,
} from '~/file-prober'
import {
  HTTPServer,
  MultipartBody,
  StringBody,
} from '~/http-server'
import {
  IChangeset,
} from '../schema'
import {
  ArhivDB,
} from '../primary/db'
import {
  isValidAuth,
  extractTokenCookie,
  resolveAsset,
  createToken,
} from './utils'

const log = createLogger('arhiv-server')

export interface IArhivServerConfig {
  readonly port: number
  readonly password: string
}

export async function createServer(db: ArhivDB, config: IArhivServerConfig, staticDirs: string[]) {
  const queue = new Queue()

  const tmpDir = await createTempDir()
  log.debug(`temp dir for files: ${tmpDir}`)
  const server = new HTTPServer(tmpDir)

  log.debug('static dirs: ', staticDirs.join(', '))

  server.use(async function authMiddleware(context, next) {
    const { req, res } = context

    res.headers['Referrer-Policy'] = 'no-referrer'

    const isAuthorized = isValidAuth(extractTokenCookie(req.headers.cookie || ''), config.password)
    if (!isAuthorized
      && req.url.pathname!.startsWith('/api')
      && req.url.pathname !== '/api/auth') {
      res.statusCode = 403

      return
    }

    await next()
  })

  // POST /api/auth
  server.post(pm`/api/auth`, async ({ req, res }) => {
    const body = req.body!
    if (!(body instanceof StringBody)) {
      res.statusCode = 415

      return
    }

    if (body.value !== config.password) {
      res.statusCode = 401

      return
    }

    res.headers['set-cookie'] = createToken(config.password)
  })

  // POST /api/changeset
  server.post(pm`/api/changeset`, async ({ res, req }) => {
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

    const assets: Dict = {}
    for (const file of body.files) {
      assets[file.field] = file.file
    }

    res.body = await queue.push(() => db.applyChangeset(changeset, assets))
  })

  // GET /api/file/:fileId
  server.get(pm`/api/file/${'fileId'}`, async ({ res }, { fileId }) => {
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
  server.get(pm`/api/${'*'}`, ({ res }) => {
    res.statusCode = 400
  })

  // Handle assets + html5 history fallback
  // GET /*
  server.get(pm`/${'*'}`, async ({ res }, { '*': fileName }) => {
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
    async start() {
      await server.start(config.port)
      log.info(`listening on http://localhost:${config.port}`)
    },

    async stop() {
      rmrfSync(tmpDir)

      return Promise.all([server.stop(), queue.close()])
    },
  }
}
