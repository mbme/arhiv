import { test } from '~/tester'
import {
  trimLeft,
  camelCase2kebabCase,
} from './string'

test('trimLeft', (assert) => {
  assert.equal(trimLeft(' *test', ' *'), 'test')
  assert.equal(trimLeft(' *', ' *'), '')
  assert.equal(trimLeft('test'), 'test')
  assert.equal(trimLeft(' test '), 'test ')
})

test('camelCase2kebabCase', (assert) => {
  assert.equal(camelCase2kebabCase('font'), 'font')
  assert.equal(camelCase2kebabCase('fontSize'), 'font-size')
  assert.equal(camelCase2kebabCase('fontSizeLong'), 'font-size-long')
})
