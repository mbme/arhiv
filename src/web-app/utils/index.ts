import { aesEncrypt, text2buffer, sha256 } from '../../utils/browser'

export async function authorize(password: string) {
  const token = await aesEncrypt(`valid ${Date.now()}`, await sha256(text2buffer(password)))
  document.cookie = `token=${token}; path=/`
}

export async function deauthorize() {
  document.cookie = 'token=0; path=/'
}
