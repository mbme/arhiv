import { getRandomBytes } from './platform.js'

const readUInt32 = (bytes: Uint8Array) => new DataView(bytes.buffer).getUint32(0)

const MAX_RANGE = 2 ** 32
// [min, max], max > min
export function randomInt(min: number, max: number): number {
  const range = max - min

  if (range <= 0) {
    throw new Error(`max must be greater than min, got [${min}, ${max}]`)
  }

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

export function shuffle<T>(array: T[]): T[] {
  const result = array.slice(0)

  if (array.length < 2) {
    return result
  }

  for (let i = 0; i < array.length; i += 1) {
    const index = randomInt(0, array.length - 1)

    // swap items
    const item = result[index]
    result[index] = result[i]
    result[i] = item
  }

  return result
}

export const pickRandomItem = <T>(arr: ArrayLike<T>) => arr[randomInt(0, arr.length - 1)]

export function randomId(alphabet: string, size: number) {
  let id = ''

  for (let i = 0; i < size; i += 1) {
    id += pickRandomItem(alphabet)
  }

  return id
}
