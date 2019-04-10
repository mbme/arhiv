import {
  zeroOrMore,
  orElse,
  regex,
  eof,
  everythingUntil,
  setLabel,
} from '~/parser-combinator'

export const newlines = setLabel(regex(/^\n{2,}/), 'newlines')

export const paragraph = setLabel(
  everythingUntil(
    orElse(
      eof,
      newlines,
    ),
  ),
  'paragraph',
)

export const markupParser = zeroOrMore(orElse(newlines, paragraph))
