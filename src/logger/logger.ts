/* tslint:disable:no-console */
import { ChronoFormatter } from '~/chrono'
import {
  TermColor,
  termColors,
  ColorCode,
} from '~/utils/term-colors'
import {
  LogLevel,
  LEVELS,
} from './types'
import { config } from './config'

const dateFormat = new ChronoFormatter('YYYY-MM-DD HH:mm:ss,SSS')

export class Logger {
  private _namespaceColor?: ColorCode

  constructor(
    private _namespace: string,
    _namespaceColor?: TermColor,
  ) {
    if (_namespaceColor) {
      this._namespaceColor = termColors[_namespaceColor]
    }
  }

  private _getNamespace() {
    let content = this._namespace.length > config.namespaceSize
      ? '~' + this._namespace.substring(this._namespace.length - config.namespaceSize + 1)
      : this._namespace.padStart(config.namespaceSize)

    if (this._namespaceColor) {
      content = this._namespaceColor(content)
    }

    return `[${content}]`
  }

  private _getDate() {
    if (config.includeDateTime) {
      return dateFormat.format(new Date())
    }

    return ''
  }

  private _log(level: LogLevel, msg: string, ...params: any[]) {
    if (LEVELS[level] < LEVELS[config.minLogLevel]) {
      return
    }

    const logMessage = [
      this._getDate(),
      this._getNamespace(),
      level.padEnd(5),
      msg,
    ].join(' ')

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
}
