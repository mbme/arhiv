export const LEVELS = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
}

export type LogLevel = keyof typeof LEVELS
