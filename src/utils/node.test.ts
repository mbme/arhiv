import {
  test,
  asserts,
} from '~/tester'
import {
  aesEncrypt,
  aesDecrypt,
} from './node'

test('Encrypt/decrypt', () => {
  const text = 'Some great text: with a colon'
  const password = 'Giant password'
  asserts.equal(aesDecrypt(aesEncrypt(text, password), password), text)
})
