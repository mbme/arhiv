export function stringifyChunks(chunks: Buffer[]) {
  return Buffer.concat(chunks).toString('utf-8')
}
