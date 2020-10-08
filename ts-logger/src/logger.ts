/* eslint-disable no-console */
import { ChronoFormatter } from '@v/chrono'
import {
  TermColor,
  termColors,
  ColorCode,
} from '@v/utils'
import {
  LogLevel,
  LEVELS,
} from './types'
import { config } from './config'

const dateFormat = new ChronoFormatter('YYYY-MM-DD HH:mm:ss,SSS')
function getDate() {
  if (config.includeDateTime) {
    return dateFormat.format(new Date())
  }

  return ''
}

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

  private _log(level: LogLevel, msg: string, ...params: any[]) {
    if (LEVELS[level] < LEVELS[config.minLogLevel]) {
      return
    }

    const logMessage = [
      getDate(),
      this._getNamespace(),
      config.includeLogLevel && level.padEnd(5),
      msg,
    ].filter(Boolean).join(' ')

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
    }
  }

  debug = this._log.bind(this, 'DEBUG')

  info = this._log.bind(this, 'INFO')

  warn = this._log.bind(this, 'WARN')

  error = this._log.bind(this, 'ERROR')
}
