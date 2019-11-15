import path from 'path'
import {
  createLogger,
  setLogLevelStr,
} from '~/utils'
import {
  rmrfSync,
} from '~/utils/fs'
import {
  createRunnable,
  onTermination,
} from '~/utils/runnable'
import { getFakeNotes } from './faker'
import {
  PrimaryDB,
  PrimaryFSStorage,
} from '../isodb/primary'
import createServer from '../isodb/server/server'

const isProduction = process.env.NODE_ENV === 'production'

setLogLevelStr(process.env.LOG || '')

const log = createLogger('isodb-server')

createRunnable(async (port: string, password: string, storageDir: string, ...args: string[]) => {
  if (!port || !password || !storageDir) {
    throw new Error('port, password & storageDir are required')
  }

  const rootDir = process.cwd()

  const storage = await PrimaryFSStorage.create(storageDir)
  onTermination(() => storage.stop())

  const db = new PrimaryDB(storage)

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
