import { TermColor } from '~/utils/term-colors'
import { Logger } from './logger'
import {
  LogLevel,
  LEVELS,
} from './types'

export { ILoggerConfig } from './types'

export { config as loggerConfig } from './config'


export function createLogger(namespace: string, namespaceColor?: TermColor) {
  return new Logger(namespace, namespaceColor)
}

const isLogLevel = (lvl: string): lvl is LogLevel => Object.keys(LEVELS).includes(lvl)

export function parseLogLevel(levelStr: string, fallbackLvl?: LogLevel): LogLevel {
  if (isLogLevel(levelStr)) {
    return levelStr
  }

  if (fallbackLvl) {
    return fallbackLvl
  }

  throw new Error(`unexpected log level: ${levelStr}`)
}
