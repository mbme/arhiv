import path from 'path'
import {
  createLogger,
  loggerConfig,
  parseLogLevel,
} from '~/logger'
import {
  rmrfSync,
} from '~/utils/fs'
import {
  createRunnable,
  onTermination,
} from '~/utils/runnable'
import { getFakeNotes } from './faker'
import {
  ArhivDB,
  FSStorage,
} from './db'
import createServer from './server'

const isProduction = process.env.NODE_ENV === 'production'

loggerConfig.minLogLevel = parseLogLevel(process.env.LOG || '')
loggerConfig.includeDateTime = true

const log = createLogger('arhiv-server')

createRunnable(async (port: string, password: string, storageDir: string, ...args: string[]) => {
  if (!port || !password || !storageDir) {
    throw new Error('port, password & storageDir are required')
  }

  const rootDir = process.cwd()

  const storage = await FSStorage.open(storageDir, args.includes('--init'))
  onTermination(() => storage.stop())

  const db = new ArhivDB(storage)

  if (!isProduction && args.includes('--gen-data')) {
    try {
      const resourcesDir = path.join(rootDir, 'resources')

      const {
        documents,
        attachments,
        attachedFiles,
        tempDir,
      } = await getFakeNotes(resourcesDir, 30)

      await db.applyChangeset({
        schemaVersion: storage.getSchemaVersion(),
        baseRev: 0,
        documents,
        attachments,
      }, attachedFiles).finally(() => rmrfSync(tempDir))

      log.info(`Generated ${documents.length} fake notes`)
    } catch (e) {
      log.error('Failed to generate fake notes', e)
      process.exit(1)
    }
  }

  const server = createServer(db, password, [
    path.join(rootDir, 'src/web-app/static'),
    path.join(rootDir, 'tsdist/web-app-src'),
  ])

  await server.start(parseInt(port, 10))
  log.info(`listening on http://localhost:${port}`)

  onTermination(async () => {
    log.info(`stopping...`)
    try {
      await server.stop()
      process.exit(0)
    } catch (e) {
      log.error('failed to stop', e)
      process.exit(1)
    }
  })
})
