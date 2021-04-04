import {
  test,
  assertEqual,
  assertThrows,
} from '@v/tester'
import {
  trimLeft,
  camelCase2kebabCase,
  trimPrefix,
  trimSuffix,
  countSubstring,
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

test('trimPrefix', () => {
  assertEqual(trimPrefix('test', '/'), 'test')
  assertEqual(trimPrefix('/test', '/'), 'test')
  assertEqual(trimPrefix('//test', '/'), '/test')
  assertEqual(trimPrefix('/test', '/te'), 'st')
})

test('trimSuffix', () => {
  assertEqual(trimSuffix('test', '/'), 'test')
  assertEqual(trimSuffix('test/', '/'), 'test')
  assertEqual(trimSuffix('test//', '/'), 'test/')
  assertEqual(trimSuffix('test/', 'st/'), 'te')
})

test('countSubstring', () => {
  assertEqual(countSubstring('test', 'tes'), 1)
  assertEqual(countSubstring('', 'tes'), 0)
  assertThrows(() => {
    countSubstring('test', '')
  })
  assertEqual(countSubstring('testest', 'test'), 1)
  assertEqual(countSubstring('"test"est', '"'), 2)
})
