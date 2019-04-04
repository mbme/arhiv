import {
  zeroOrMore,
  orElse,
  regex,
  eof,
  everythingUntil,
  setLabel,
} from '~/parser-combinator'

const newlines = setLabel(regex(/\n{2,}/), 'newlines')

const paragraph = setLabel(
  everythingUntil(
    orElse(
      eof,
      newlines,
    ),
  ),
  'paragraph',
)

const markupParser = zeroOrMore(orElse(newlines, paragraph))
