import path from 'path'
import {
  createLogger,
  setLogLevelStr,
} from '~/utils'
import {
  createTempDir,
  rmrfSync,
} from '~/utils/fs'
import { createRunnable } from '~/utils/runnable'
import { getFakeNotes } from './faker'
import {
  PrimaryDB,
  PrimaryInMemStorage,
} from '../isodb/primary'
import createServer from '../isodb/server/server'

const isProduction = process.env.NODE_ENV === 'production'

const STATIC_DIR = path.join(process.env.BASE_DIR!, 'src/web-app/static')
const DIST_DIR = path.join(process.env.BASE_DIR!, 'dist')

setLogLevelStr(process.env.LOG || '')

const log = createLogger('isodb-server')

createRunnable(async (port: string, password: string, rootDir: string, ...args: string[]) => {
  if (!port || !password || !rootDir) {
    throw new Error('port, password & rootDir are required')
  }

  const storageTempDir = await createTempDir()
  const db = new PrimaryDB(new PrimaryInMemStorage(storageTempDir))

  if (!isProduction && args.includes('--gen-data')) {
    const {
      documents,
      attachments,
      attachedFiles,
      tempDir,
    } = await getFakeNotes(30)

    await db.applyChangeset({
      baseRev: 0,
      documents,
      attachments,
    }, attachedFiles)

    rmrfSync(tempDir)

    log.info(`Generated ${documents.length} fake notes`)
  }

  const server = createServer(db, password, [STATIC_DIR, DIST_DIR])

  await server.start(parseInt(port, 10))
  log.info(`listening on http://localhost:${port}`)

  async function close(signal: NodeJS.Signals) {
    log.simple()
    log.info(`Got signal ${signal}, stopping`)
    try {
      await server.stop()
      rmrfSync(storageTempDir)
      process.exit(0)
    } catch (e) {
      log.error('failed to stop', e)
      process.exit(1)
    }
  }

  process.on('SIGINT', close)
  process.on('SIGTERM', close)
})
