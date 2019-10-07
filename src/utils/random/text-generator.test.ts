import {
  test,
  asserts,
} from '~/tester'
import { getWords } from './text-generator'

test('getWords', () => {
  asserts.deepEqual(getWords('Split it, not; dr. go!'), ['split', 'it', ',', 'not', ';', 'dr.', 'go', '!'])
})
