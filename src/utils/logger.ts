/* tslint:disable:no-console */
import { formatDate } from './date'

const LEVELS = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
}

type LogLevel = keyof typeof LEVELS

// tslint:disable-next-line:no-unsafe-any
const isLogLevel = (lvl: any): lvl is LogLevel => Object.keys(LEVELS).includes(lvl)

let minLogLevel: LogLevel = 'INFO'

class Logger {
  constructor(public namespace: string) { }

  private _log(level: LogLevel, msg: string, ...params: any[]) {
    if (LEVELS[level] < LEVELS[minLogLevel]) return

    const args = [
      `${formatDate(new Date())} ${this.namespace ? `[${this.namespace}]` : ''} ${level.padEnd(5)} ${msg}`,
      ...params,
    ]

    switch (level) {
      case 'DEBUG':
        console.debug(...args)
        break
      case 'INFO':
        console.info(...args)
        break
      case 'WARN':
        console.warn(...args)
        break
      case 'ERROR':
        console.error(...args)
        break
      default:
        throw new Error(`Wrong level ${level}`)
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
  } else if (fallbackLvl) {
    setLogLevel(fallbackLvl)
  }
}

export const log = new Logger('')
