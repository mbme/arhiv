/* tslint:disable:no-console */
import { format as formatDate } from 'date-fns'

enum LEVEL {
  DEBUG,
  INFO,
  WARN,
  ERROR,
}

const PRIORITY = {
  [LEVEL.DEBUG.toString()]: 0,
  [LEVEL.INFO.toString()]: 1,
  [LEVEL.WARN.toString()]: 2,
  [LEVEL.ERROR.toString()]: 3,
}

const minLogLevel = process.env.LOG || LEVEL.INFO
if (!Object.values(LEVEL).includes(minLogLevel)) throw new Error(`Illegal log level ${minLogLevel}`)

function createLevelLogger(level: LEVEL, namespace: string) {
  const name = level.toString().padEnd(5)

  return (msg: string, ...params: any[]) => { // tslint:disable-line no-any
    if (PRIORITY[level] < PRIORITY[minLogLevel]) return

    const args = [
      `${formatDate(new Date(), 'YYYY-MM-DD HH:mm:ss,SSS')} ${namespace ? `[${namespace}]` : ''} ${name} ${msg}`,
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
