/* tslint:disable:no-unsafe-any */
import path from 'path'
import { exec } from '../utils/node'

// TODO images, video

const SUPPORTED_MEDIA_FORMATS = ['mp3']

async function probeMediaFileMeta(filePath: string) {
  const { format } = JSON.parse(await exec(`ffprobe -v quiet -of json -show_format -i ${filePath}`).catch((e) => {
    if (e.code !== 1) throw e

    return e.stdout
  }))

  if (!format || !SUPPORTED_MEDIA_FORMATS.includes(format.format_name)) return undefined

  return {
    bitRate: parseInt(format.bit_rate, 10),
    duration: parseFloat(format.duration),
  }
}

export default async function probeMetadata(filePath: string) {
  const mediaMeta = await probeMediaFileMeta(filePath)
  if (mediaMeta) return mediaMeta

  return {}
}

const MIME: { [key: string]: string } = {
  '.css': 'text/css',
  '.html': 'text/html',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
}
export const getMimeType = async (filePath: string) => MIME[path.extname(filePath)] || exec(`file -b -i ${filePath}`)
