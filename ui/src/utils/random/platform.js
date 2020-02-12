import crypto from 'crypto'

export function getRandomBytes(bytes) {
  if (process.env.__BROWSER__) {
    // this is window.crypto
    return crypto.getRandomValues(new Uint8Array(bytes))
  }

  // this is Node's crypto
  return crypto.randomBytes(bytes)
}
