import {
  test,
  assertDeepEqual,
  assertFalse,
} from '~/tester'
import { ArgsParser } from './args-parser'

test('allows to specify commands', () => {
  const x = ArgsParser.create().command({})

  // test if fails when no command provided
})

test('allows to specify options', () => {
  // fails on unknown options
})

test('allows to specify options demanding values', () => {
  // fails on options without value
})

test('generates help', () => {
})
