export const CRLF = '\r\n' // https://tools.ietf.org/html/rfc7230#section-3

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

const boundaryRegexp = /boundary=(.*)/
export function extractBoundary(contentTypeHeader: string): string {
  return boundaryRegexp.exec(contentTypeHeader)?.[1] || ''
}
