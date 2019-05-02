import { test } from '~/tester'
import { trimLeft } from './string'

test('trimLeft', (assert) => {
  assert.equal(trimLeft(' *test', ' *'), 'test')
  assert.equal(trimLeft(' *', ' *'), '')
  assert.equal(trimLeft('test'), 'test')
  assert.equal(trimLeft(' test '), 'test ')
})
