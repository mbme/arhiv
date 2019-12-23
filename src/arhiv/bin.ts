import path from 'path'
import {
  createLogger,
  configureLogger,
} from '~/logger'
import { rmrfSync } from '~/utils/fs'
import { getFakeNotes } from './tools/faker'
import {
  ArhivDB,
  FSStorage,
} from './primary'
import { createServer } from './server'
import { readConfig } from './tools/config'

const isProduction = process.env.NODE_ENV === 'production'

const log = createLogger('arhiv')

createRunnable(async (args, onExit) => {
  const rootDir = process.cwd() // FIXME use file location instead

  const config = await readConfig() // FIXME config should be located in the repository
  configureLogger(config.log)

  const storage = await FSStorage.open(config.storageDir, args.includes('--init'))
  onExit(() => storage.stop())

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

  const server = await createServer(db, config.server, [
    path.join(rootDir, 'src/web-app/static'),
    path.join(rootDir, 'tsdist/web-app-src'),
  ])

  await server.start()

  onExit(async () => {
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
