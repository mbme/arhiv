import {
  expect,
  anyChar,
  isFailure,
  ParserResult,
} from '@v/parser-combinator'
import { isFunction } from '@v/utils'

const pad0 = (s: string, maxLength: number) => s.padStart(maxLength, '0')

const YYYY = expect('YYYY').map(() => (date: Date) => date.getFullYear().toString())
const MM = expect('MM').map(() => (date: Date) => pad0((date.getMonth() + 1).toString(), 2))
const M = expect('M').map(() => (date: Date) => (date.getMonth() + 1).toString())
const DD = expect('DD').map(() => (date: Date) => pad0(date.getDate().toString(), 2))
const HH = expect('HH').map(() => (date: Date) => pad0(date.getHours().toString(), 2))
const mm = expect('mm').map(() => (date: Date) => pad0(date.getMinutes().toString(), 2))
const ss = expect('ss').map(() => (date: Date) => pad0(date.getSeconds().toString(), 2))
const SSS = expect('SSS').map(() => (date: Date) => pad0(date.getSeconds().toString(), 3))

const patternParser = YYYY
  .orElse(MM)
  .orElse(M)
  .orElse(DD)
  .orElse(HH)
  .orElse(mm)
  .orElse(ss)
  .orElse(SSS)
  .orElse(anyChar)
  .oneOrMore()

export class ChronoFormatter {
  private _pattern: ParserResult<typeof patternParser>

  constructor(pattern: string) {
    const result = patternParser.parseAll(pattern)

    if (isFailure(result)) {
      throw new Error(`Failed to parse chrono pattern "${pattern}"`)
    }

    this._pattern = result.value
  }

  format(date: Date): string {
    return this._pattern.map((node) => {
      if (isFunction(node)) {
        return node(date)
      }

      return node
    }).join('')
  }
}
