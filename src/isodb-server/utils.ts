import path from 'path'
import fs from 'fs'
import http from 'http'
import Busboy from 'busboy'
import * as utils from '../utils/node'
import { listFiles } from '../fs/utils'
import { ILazy } from '../utils'

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

// Extract action & assets from multipart/form-data POST request
export function readFormData(tmpDir: ILazy<Promise<string>>, req: http.IncomingMessage) {
  const assets: { [key: string]: string } = {}
  const fields: { [key: string]: string } = {}

  const busboy = new Busboy({ headers: req.headers })

  busboy.on('file', async (fieldName, fileStream) => {
    if (assets[fieldName]) {
      throw new Error(`request contains duplicate file "${fieldName}"`)
    }

    const asset = path.join(await tmpDir.value, fieldName)
    assets[fieldName] = asset
    fileStream.pipe(fs.createWriteStream(asset))
  })

  busboy.on('field', (fieldName, val) => {
    if (fields[fieldName]) {
      throw new Error(`request contains duplicate field "${fieldName}"`)
    }

    fields[fieldName] = val
  })

  return new Promise((resolve) => {
    busboy.on('finish', () => resolve({ fields, assets }))
    req.pipe(busboy)
  })
}
