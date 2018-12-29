/* tslint:disable:no-console */
import { formatDate } from '../utils/date'

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

function createLevelLogger(level: string, namespace: string) {
  const name = level.padEnd(5)

  return (msg: string, ...params: any[]) => { // tslint:disable-line no-any
    if (PRIORITY[level] < PRIORITY[minLogLevel]) return

    const args = [
      `${formatDate(new Date())} ${namespace ? `[${namespace}]` : ''} ${name} ${msg}`,
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
}

export function createLogger(namespace: string) {
  return {
    debug: createLevelLogger(LEVEL.DEBUG, namespace),
    info: createLevelLogger(LEVEL.INFO, namespace),
    warn: createLevelLogger(LEVEL.WARN, namespace),
    error: createLevelLogger(LEVEL.ERROR, namespace),

    simple(...params: any[]) { // tslint:disable-line no-any
      console.log(...params)
    },
  }
}

export default createLogger('')
