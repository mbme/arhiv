import {
  zeroOrMore,
  orElse,
  regex,
  eof,
  everythingUntil,
  setLabel,
  satisfy,
  oneOrMore,
  andThen,
} from '~/parser-combinator'
import { oneOf } from 'prop-types';

export const newlines = setLabel(regex(/^\n{2,}/), 'newlines')
export const bold = setLabel(regex(/^\*.*\*/), 'bold')
export const mono = setLabel(regex(/^`.*`/), 'mono')
const anyChar = setLabel(satisfy(str => [true, str[0]]), 'char')

const line = setLabel(
  orElse(
    bold,
    mono,

    anyChar,
  ),
  'line',
)

export const paragraph = setLabel(
  andThen(
    oneOrMore(
      orElse(
        bold,
        mono,
        anyChar,
      )
    ),
    orElse(
      eof,
      newlines,
    ),
  ),
  'paragraph',
)

export const markupParser = zeroOrMore(orElse(newlines, paragraph))
