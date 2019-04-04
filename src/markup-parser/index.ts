import {
  zeroOrMore,
  orElse,
  regex,
  eof,
  everythingUntil,
} from '~/parser-combinator'

const newlines = regex(/\n{2,}/)

const paragraph = everythingUntil(orElse(eof, newlines))

const markupParser = zeroOrMore(orElse(newlines, paragraph))
