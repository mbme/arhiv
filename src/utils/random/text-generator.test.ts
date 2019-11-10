import {
  test,
  assert,
} from '~/tester'
import { getWords } from './text-generator'

test('getWords', () => {
  assert.deepEqual(getWords('Split it, not; dr. go!'), ['split', 'it', ',', 'not', ';', 'dr.', 'go', '!'])
})
