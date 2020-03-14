import {
  test,
  assertDeepEqual,
} from '@v/tester'
import { getWords } from './text-generator'

test('getWords', () => {
  assertDeepEqual(getWords('Split it, not; dr. go!'), ['split', 'it', ',', 'not', ';', 'dr.', 'go', '!'])
})
