import { TermColor } from '~/utils/term-colors'
import { Logger } from './logger'

export { ILoggerConfig } from './types'

export {
  config as loggerConfig,
  configureLogger,
} from './config'

export function createLogger(namespace: string, namespaceColor?: TermColor) {
  return new Logger(namespace, namespaceColor)
}
