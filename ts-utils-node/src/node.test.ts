import {
  test,
  assertEqual,
} from '@v/tester'
import {
  aesEncrypt,
  aesDecrypt,
} from './node'

test('Encrypt/decrypt', () => {
  const text = 'Some great text: with a colon'
  const password = 'Giant password'

  assertEqual(aesDecrypt(aesEncrypt(text, password), password), text)
})
