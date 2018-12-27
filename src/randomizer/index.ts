const getRandomBytes: (bytes: number) => Uint8Array = __SERVER__
  // tslint:disable-next-line:no-var-requires
  ? require('crypto').randomBytes
  : (bytes: number) => window.crypto.getRandomValues(new Uint8Array(bytes))

const readUInt32 = (bytes: Uint8Array) => new DataView(bytes.buffer).getUint32(0)

const MAX_RANGE = 2 ** 32
// [min, max]
export function randomInt(min: number, max: number) {
  const range = max - min
  if (range > MAX_RANGE) {
    throw new Error('range is too wide')
  }
  const maxSample = Math.floor(MAX_RANGE / range) * range

  let sample
  do {
    sample = readUInt32(getRandomBytes(4))
  } while (sample > maxSample)

  return min + (sample % range)
}

export function shuffle<T>(array: T[]) {
  const result = array.slice(0)

  for (let i = 0; i < array.length; i += 1) {
    const index = randomInt(i, array.length - 1)

    // swap items
    const item = result[index]
    result[index] = result[i]
    result[i] = item
  }

  return result
}

export const randomArrValue = <T>(arr: ArrayLike<T>) => arr[randomInt(0, arr.length - 1)]

export function randomId(alphabet: string, size: number) {
  let id = ''

  for (let i = 0; i < size; i += 1) {
    id += randomArrValue(alphabet)
  }

  return id
}
