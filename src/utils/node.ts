import childProcess from 'child_process'
import crypto from 'crypto'
import fs from 'fs'
import readline from 'readline'
import { promisify } from 'util'
import zlib from 'zlib'

export const hash = (hashType: string) => (
  (value: string, buffer = false) => crypto.createHash(hashType).update(value).digest(buffer ? undefined as any : 'hex')
)
export const sha256 = hash('sha256')

export function sha256File(filePath: string): Promise<string> {
  return new Promise(
    (resolve, reject) => fs.createReadStream(filePath)
      .on('error', reject)
      .pipe(crypto.createHash('sha256').setEncoding('hex'))
      .on('finish', function onFinish(this: fs.ReadStream) {
        resolve(this.read() as string)
      }),
  )
}

export function aesEncrypt(text: string, password: string) {
  const iv = crypto.randomBytes(16) // always 16 for AES

  const cipher = crypto.createCipheriv('aes-256-cbc', sha256(password, true), iv)

  return iv.toString('hex') + ':' + cipher.update(text, 'utf8', 'hex') + cipher.final('hex')
}

export function aesDecrypt(text: string, password: string) {
  const [iv, encryptedText] = text.split(':')

  const decipher = crypto.createDecipheriv('aes-256-cbc', sha256(password, true), Buffer.from(iv, 'hex'))

  return decipher.update(encryptedText, 'hex', 'utf8') + decipher.final('utf8')
}

export function readStream(stream: NodeJS.ReadableStream): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const body: Uint8Array[] = []
    stream.on('data', (chunk: Uint8Array) => body.push(chunk))
    stream.on('end', () => resolve(Buffer.concat(body)))
    stream.on('error', reject)
  })
}
export async function readStreamAsString(stream: NodeJS.ReadableStream): Promise<string> {
  const data = await readStream(stream)

  return data.toString('utf8')
}

export const gzip = promisify(zlib.gzip)

export function ask(question: string): Promise<string> {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout })

  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      resolve(answer)
      rl.close()
    })
  })
}

export const execRaw = promisify(childProcess.exec)

export function exec(command: string): Promise<string> {
  return execRaw(command).then(({ stdout }) => stdout.trim())
}

class SpawnError extends Error {
  constructor(
    public readonly code: number,
    public readonly result: string,
  ) {
    super()
  }
}
export function spawn(command: string, ...args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const process = childProcess.spawn(command, args)

    let result = ''

    process.stdout.on('data', (data) => {
      result += data
    })

    process.on('close', (code) => {
      if (code) {
        reject(new SpawnError(code, result))
      } else {
        resolve(result.trim())
      }
    })
    process.on('error', reject)
  })
}

export function pipePromise(from: NodeJS.ReadableStream, to: NodeJS.WritableStream): Promise<void> {
  return new Promise((resolve, reject) => {
    from
      .on('end', resolve)
      .on('error', reject)
      .pipe(to)
  })
}
