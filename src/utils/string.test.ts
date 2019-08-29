import {
  test,
  asserts,
} from '~/tester'
import {
  trimLeft,
  camelCase2kebabCase,
} from './string'

test('trimLeft', () => {
  asserts.equal(trimLeft(' *test', ' *'), 'test')
  asserts.equal(trimLeft(' *', ' *'), '')
  asserts.equal(trimLeft('test'), 'test')
  asserts.equal(trimLeft(' test '), 'test ')
})

test('camelCase2kebabCase', () => {
  asserts.equal(camelCase2kebabCase('font'), 'font')
  asserts.equal(camelCase2kebabCase('fontSize'), 'font-size')
  asserts.equal(camelCase2kebabCase('fontSizeLong'), 'font-size-long')
})
