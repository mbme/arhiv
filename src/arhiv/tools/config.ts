import {
  ILoggerConfig,
  createLogger,
} from '~/logger'
import { readJSON } from '~/utils/fs'
import { IArhivServerConfig } from '../server'

const log = createLogger('arhiv-config')

export interface IArhivConfig {
  readonly server: IArhivServerConfig
  readonly log: ILoggerConfig
  readonly storageDir: string
}

export function readConfig(path: string = '~/.config/arhiv.json'): Promise<IArhivConfig> {
  log.debug(`reading config from ${path}`)

  return readJSON<IArhivConfig>(path)
}
