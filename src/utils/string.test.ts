import {
  test,
  assertEqual,
} from '~/tester'
import {
  trimLeft,
  camelCase2kebabCase,
} from './string'

test('trimLeft', () => {
  assertEqual(trimLeft(' *test', ' *'), 'test')
  assertEqual(trimLeft(' *', ' *'), '')
  assertEqual(trimLeft('test'), 'test')
  assertEqual(trimLeft(' test '), 'test ')
})

test('camelCase2kebabCase', () => {
  assertEqual(camelCase2kebabCase('font'), 'font')
  assertEqual(camelCase2kebabCase('fontSize'), 'font-size')
  assertEqual(camelCase2kebabCase('fontSizeLong'), 'font-size-long')
})
