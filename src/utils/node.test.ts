import {
  test,
  assert,
} from '~/tester'
import {
  aesEncrypt,
  aesDecrypt,
} from './node'

test('Encrypt/decrypt', () => {
  const text = 'Some great text: with a colon'
  const password = 'Giant password'
  assert.equal(aesDecrypt(aesEncrypt(text, password), password), text)
})
