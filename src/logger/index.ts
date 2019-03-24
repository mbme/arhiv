/* tslint:disable:no-console */
import { formatDate } from '~/utils'

const LEVEL = {
  DEBUG: 'DEBUG',
  INFO: 'INFO',
  WARN: 'WARN',
  ERROR: 'ERROR',
}

const PRIORITY = {
  [LEVEL.DEBUG]: 0,
  [LEVEL.INFO]: 1,
  [LEVEL.WARN]: 2,
  [LEVEL.ERROR]: 3,
}

const minLogLevel = process.env.LOG || LEVEL.INFO
if (!Object.values(LEVEL).includes(minLogLevel)) throw new Error(`Illegal log level ${minLogLevel}`)

class Logger {
  constructor(public namespace: string) { }

  _log(level: string, msg: string, ...params: any[]) {
    if (PRIORITY[level] < PRIORITY[minLogLevel]) return

    const args = [
      `${formatDate(new Date())} ${this.namespace ? `[${this.namespace}]` : ''} ${level.padEnd(5)} ${msg}`,
      ...params,
    ]

    switch (level) {
      case LEVEL.DEBUG:
        console.debug(...args)
        break
      case LEVEL.INFO:
        console.info(...args)
        break
      case LEVEL.WARN:
        console.warn(...args)
        break
      case LEVEL.ERROR:
        console.error(...args)
        break
      default:
        throw new Error(`Wrong level ${level}`)
    }
  }

  debug = this._log.bind(this, LEVEL.DEBUG)
  info = this._log.bind(this, LEVEL.INFO)
  warn = this._log.bind(this, LEVEL.WARN)
  error = this._log.bind(this, LEVEL.ERROR)

  simple(...params: any[]) {
    console.log(...params)
  }
}

export function createLogger(namespace: string) {
  return new Logger(namespace)
}

export default new Logger('')
