export function stringifyChunks(chunks: Buffer[]) {
  return Buffer.concat(chunks).toString('utf-8')
}

export function isSubarrayAt(array: Buffer, pos: number, subarray: Buffer) {
  if (pos + subarray.byteLength > array.byteLength) {
    return false
  }

  const result = array.compare(subarray, 0, subarray.byteLength, pos, pos + subarray.byteLength)

  return result === 0
}
