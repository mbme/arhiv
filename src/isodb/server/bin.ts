import path from 'path'
import {
  createLogger,
  setLogLevelStr,
} from '~/utils'
import { getFakeNotes } from '~/randomizer/faker'
import {
  PrimaryDB,
  PrimaryInMemStorage,
} from '../primary'
import createServer from './server'

const isProduction = process.env.NODE_ENV === 'production'

const STATIC_DIR = path.join(process.env.BASE_DIR!, 'src/web-app/static')
const DIST_DIR = path.join(process.env.BASE_DIR!, 'dist')

setLogLevelStr(process.env.LOG || '')

const log = createLogger('isodb-server')

export default async function run(port: string, password: string, rootDir: string, ...args: string[]) {
  if (!port || !password || !rootDir) throw new Error('port, password & rootDir are required')

  const db = new PrimaryDB(new PrimaryInMemStorage())
  if (!isProduction && args.includes('--gen-data')) {
    const {
      documents,
      attachments,
      attachedFiles,
    } = await getFakeNotes(30)

    await db.applyChangeset({
      baseRev: 0,
      documents,
      attachments,
    }, attachedFiles)

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
      process.exit(0)
    } catch (e) {
      log.error('failed to stop', e)
      process.exit(1)
    }
  }

  process.on('SIGINT', close)
  process.on('SIGTERM', close)
}
