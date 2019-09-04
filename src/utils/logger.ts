/* tslint:disable:no-console */
import { formatDate } from './date'

const LEVELS = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
}

type LogLevel = keyof typeof LEVELS

const isLogLevel = (lvl: string): lvl is LogLevel => Object.keys(LEVELS).includes(lvl)

let minLogLevel: LogLevel = 'INFO'

const MAX_NAMESPACE_SIZE = 20

class Logger {
  constructor(private _namespace: string) { }

  private _getNamespace() {
    if (this._namespace.length > MAX_NAMESPACE_SIZE) {
      return '~' + this._namespace.substring(this._namespace.length - MAX_NAMESPACE_SIZE + 1)
    }

    return this._namespace.padStart(MAX_NAMESPACE_SIZE)
  }

  private _log(level: LogLevel, msg: string, ...params: any[]) {
    if (LEVELS[level] < LEVELS[minLogLevel]) {
      return
    }

    const logMessage = `${formatDate(new Date())} [${this._getNamespace()}] ${level.padEnd(5)} ${msg}`

    switch (level) {
      case 'DEBUG': {
        console.debug(logMessage, ...params)
        break
      }
      case 'INFO': {
        console.info(logMessage, ...params)
        break
      }
      case 'WARN': {
        console.warn(logMessage, ...params)
        break
      }
      case 'ERROR': {
        console.error(logMessage, ...params)
        break
      }
      default: {
        throw new Error(`Wrong level ${level}`)
      }
    }
  }

  debug = this._log.bind(this, 'DEBUG')
  info = this._log.bind(this, 'INFO')
  warn = this._log.bind(this, 'WARN')
  error = this._log.bind(this, 'ERROR')

  simple(...params: any[]) {
    console.log(...params)
  }
}

export function createLogger(namespace: string) {
  return new Logger(namespace)
}

export function setLogLevel(level: LogLevel) {
  minLogLevel = level
}

export function setLogLevelStr(levelStr: string, fallbackLvl?: LogLevel) {
  if (isLogLevel(levelStr)) {
    setLogLevel(levelStr)

    return
  }

  if (fallbackLvl) {
    setLogLevel(fallbackLvl)

    return
  }

  // no raw level or fallback level provided, so just ignore the call
}
