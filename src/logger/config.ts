import {
  LogLevel,
} from './types'

export interface IConfig {
  minLogLevel: LogLevel
  includeDateTime: boolean
  namespaceSize: number
}

export const config: IConfig = {
  minLogLevel: 'INFO',
  includeDateTime: true,
  namespaceSize: 20,
}
