import path from 'path'
import * as utils from '~/utils/node'
import { listFiles } from '~/utils/fs'

export async function resolveAsset(dirs: string[], name: string) {
  for (const dir of dirs) {
    const files = await listFiles(dir)

    if (files.includes(name)) return path.join(dir, name)
  }

  return undefined
}

// create auth token cookie
export function createToken(password: string) {
  const token = utils.aesEncrypt(`valid ${Date.now()}`, utils.sha256(password))

  const oneHour = 1 * 60 * 60 // seconds

  return `token=${encodeURIComponent(token)}; path=/; Max-Age=${oneHour}; SameSite=Strict`
}

// extract auth token from cookies
export function extractTokenCookie(cookies: string) {
  const [tokenCookie] = cookies.split(';').filter(c => c.startsWith('token='))

  if (!tokenCookie) return ''

  return decodeURIComponent(tokenCookie.substring(6))
}

// token: AES("valid <generation timestamp>", SHA256(password))
export function isValidAuth(token: string, password: string) {
  try {
    return /^valid \d+$/.test(utils.aesDecrypt(token || '', utils.sha256(password)))
  } catch (ignored) {
    return false
  }
}
