export { ChronoFormatter } from './formatter'

export const nowS = () => Date.now() / 1000

export function dateNow(): string { // ISO8601
  return new Date().toISOString()
}
