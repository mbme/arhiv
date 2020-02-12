export function readFile(file: Blob) {
  return new Promise<Uint8Array>((resolve, reject) => {
    const reader = new FileReader()

    reader.onload = () => resolve(new Uint8Array(reader.result as ArrayBuffer))
    reader.onerror = reject

    reader.readAsArrayBuffer(file)
  })
}

export const bytesToHexString = (buffer: ArrayBuffer) => (
  Array.from(new Uint8Array(buffer)).map(b => ('00' + b.toString(16)).slice(-2)).join('')
)

export const sha256 = (buffer: ArrayBuffer) => crypto.subtle.digest('SHA-256', buffer).then(bytesToHexString)

export const text2buffer = (text: string) => new TextEncoder().encode(text)

export async function aesEncrypt(text: string, password: string) {
  const alg = { name: 'AES-CBC', iv: crypto.getRandomValues(new Uint8Array(16)) }
  const passwordHash = await crypto.subtle.digest('SHA-256', text2buffer(password))
  const key = await crypto.subtle.importKey('raw', passwordHash, alg as any, false, ['encrypt'])

  const encrypted = await crypto.subtle.encrypt(alg, key, text2buffer(text)).then(bytesToHexString)

  return bytesToHexString(alg.iv as any) + ':' + encrypted
}
