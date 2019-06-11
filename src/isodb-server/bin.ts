import path from 'path'
import { createLogger, setLogLevel } from '~/logger'
import { getFakeNotes } from '~/randomizer/faker'
import PrimaryDB from './primary'
import PrimaryInMemStorage from './primary-in-mem-storage'
import createServer from './server'

const isProduction = process.env.NODE_ENV === 'production'

const STATIC_DIR = path.join(__dirname, '../web-app/static')
const DIST_DIR = path.join(__dirname, '../../dist')

const log = createLogger('isodb-server')
setLogLevel('WARN')

export default async function run(port: string, password: string, rootDir: string, ...args: string[]) {
  if (!port || !password || !rootDir) throw new Error('port, password & rootDir are required')

  const db = new PrimaryDB(new PrimaryInMemStorage())
  if (!isProduction && args.includes('--gen-data')) {
    const {
      records,
      attachments,
      attachedFiles,
    } = await getFakeNotes(30)
    db.applyChangeset({
      baseRev: 0,
      records,
      attachments,
    }, attachedFiles)
    log.info(`Generated ${records.length} fake notes`)
  }

  const server = createServer(db, password, [STATIC_DIR, DIST_DIR])

  await server.start(parseInt(port, 10))
  log.info(`listening on http://localhost:${port}`)

  async function close() {
    // tslint:disable-next-line:no-console
    console.log()
    log.debug('stopping...')
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
