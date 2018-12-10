import path from 'path'
import fs from 'fs'
import * as utils from '../../utils/node'
import { listFiles } from '../../fs/utils'

export async function resolveAsset(dir: string, name: string) {
  if (!fs.existsSync(dir)) return undefined
  if (!await listFiles(dir).then(files => files.includes(name))) return undefined

  return path.join(dir, name)
}

// extract auth token from cookies
export function extractToken(cookies: string) {
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
