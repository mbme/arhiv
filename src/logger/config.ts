import {
  ILoggerConfig,
} from './types'

export const config: ILoggerConfig = {
  minLogLevel: 'INFO',
  includeDateTime: false,
  namespaceSize: 20,
}

export function configureLogger(patch: Partial<ILoggerConfig>) {
  for (const [key, value] of Object.entries(patch)) {
    (config as any)[key] = value
  }
}
