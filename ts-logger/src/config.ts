import {
  ILoggerConfig,
} from './types'
import { Obj } from '@v/utils'

export const config: ILoggerConfig = {
  minLogLevel: 'INFO',
  includeDateTime: false,
  includeLogLevel: false,
  namespaceSize: 20,
}

export function configureLogger(patch: Partial<ILoggerConfig>) {
  for (const [key, value] of Object.entries(patch)) {
    (config as Obj)[key] = value
  }
}
