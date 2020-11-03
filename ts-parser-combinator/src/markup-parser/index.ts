import {
  isFailure,
  isSuccess,
} from '../parser'
import { markupParser } from './parsers'
import * as nodes from './nodes'

export {
  selectLinks,
  createLink,
} from './utils'

export {
  isFailure,
  isSuccess,
  markupParser,
  nodes,
}

export function parseMarkup(source: string): nodes.NodeMarkup {
  const result = markupParser.parseAll(source)

  if (isFailure(result)) {
    throw new Error(`Failed to parse markup: ${result.toString()}`)
  }

  return result.value
}
