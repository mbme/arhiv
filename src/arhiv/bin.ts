import path from 'path'
import {
  createLogger,
  configureLogger,
  ILoggerConfig,
} from '~/logger'
import { parseInt10 } from '~/utils'
import {
  rmrfSync,
  readJSON,
  fileExists,
} from '~/utils/fs'
import {
  CliApp,
  command,
} from '~/utils/cli-app'
import { getFakeNotes } from './tools/faker'
import {
  ArhivDB,
  FSStorage,
} from './primary'
import {
  createServer,
  IArhivServerConfig,
} from './server'

const log = createLogger('arhiv')

interface IArhivConfig {
  readonly server: IArhivServerConfig
  readonly log: ILoggerConfig
  readonly storageDir: string
}

async function readConfig(): Promise<IArhivConfig> {
  const configPath = path.join(process.cwd(), 'arhiv.json')
  log.debug(`reading config from ${configPath}`)

  if (!await fileExists(configPath, true)) {
    throw new Error(`arhiv config file ${configPath} is missing`)
  }

  return readJSON<IArhivConfig>(configPath)
}

CliApp.create('arhiv')
  .addCommand(
    command('serve', 'Run arhiv server'),
    async (_, onExit) => {
      const rootDir = process.cwd()

      const config = await readConfig()
      configureLogger(config.log)

      const storage = await FSStorage.open(config.storageDir)
      onExit(() => storage.stop())

      const db = new ArhivDB(storage)

      const server = await createServer(db, config.server, [
        path.join(rootDir, 'src/web-app/static'),
        path.join(rootDir, 'tsdist/web-app-static'),
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
    },
  )
  .addCommand(
    command('init', 'Initialize arhiv data directory'),
    async () => {
      const config = await readConfig()
      configureLogger(config.log)

      const storage = await FSStorage.open(config.storageDir, true)
      await storage.stop()
    },
  )
  .addCommand(
    command('gen-data', 'Generate fake documents')
      .option('--count', 'Number of documents to generate', '30'),
    async (options, onExit) => {
      const rootDir = process.cwd()

      const config = await readConfig()
      configureLogger(config.log)

      const storage = await FSStorage.open(config.storageDir)
      onExit(() => storage.stop())

      const db = new ArhivDB(storage)

      try {
        const resourcesDir = path.join(rootDir, 'resources')

        const {
          documents,
          attachments,
          attachedFiles,
          tempDir,
        } = await getFakeNotes(resourcesDir, parseInt10(options['--count']))

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
    },
  )
  .run()
