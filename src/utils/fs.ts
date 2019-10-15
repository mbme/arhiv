import fs from 'fs'
import os from 'os'
import path from 'path'
import {
  merge,
  consumeAsyncIterable,
} from './index'

interface IGetFilesOpts {
  skipDir?: string[]
  recursive?: boolean
  fileNames?: boolean
}
const GET_FILES_OPTS = {
  skipDir: ['.git', 'node_modules'],
  recursive: true,
  fileNames: false,
}

// list files except skip dirs
export async function* getFiles(
  rootPath: string,
  options: IGetFilesOpts = GET_FILES_OPTS,
): AsyncIterableIterator<string> {
  const processedOpts = merge(GET_FILES_OPTS, options)

  for (const fileName of await fs.promises.readdir(rootPath)) {
    const filePath = path.resolve(rootPath, fileName)

    if (await isDirectory(filePath)) {
      if (!processedOpts.recursive) {
        continue
      }

      // traverse subdir only if it shouldn't be skipped
      if (processedOpts.skipDir!.includes(fileName)) {
        continue
      }

      yield* getFiles(filePath, options)
    } else {
      yield processedOpts.fileNames ? fileName : filePath
    }
  }
}

// TODO use "recursive" option of rmsync instead
export function rmrfSync(dir: string) {
  for (const file of fs.readdirSync(dir)) {
    const filePath = path.join(dir, file)

    if (fs.statSync(filePath).isDirectory()) {
      rmrfSync(filePath)
    } else {
      fs.unlinkSync(filePath)
    }
  }

  fs.rmdirSync(dir)
}

export const isFile = (filePath: string) => fs.promises.lstat(filePath).then((stats) => stats.isFile())
export const isDirectory = (filePath: string) => fs.promises.lstat(filePath).then((stats) => stats.isDirectory())

export async function listFiles(dir: string) {
  return consumeAsyncIterable(getFiles(dir, {
    recursive: false,
    fileNames: true,
  }))
}

export const createTempDir = () => fs.promises.mkdtemp(path.join(os.tmpdir(), 'v-'))

export async function withTempFiles(files: string[], cb: (tempFiles: string[]) => void | Promise<void>) {
  if (!files.length) {
    return cb([])
  }

  let dir: string | undefined
  try {
    // create temp dir
    dir = await createTempDir()

    // write temp files
    const paths = files.map((_, i) => path.join(dir!, `temp-file-${i}`))
    await Promise.all(paths.map((filePath, i) => fs.promises.writeFile(filePath, files[i])))

    await Promise.resolve(cb(paths))
  } finally { // do cleanup in any case
    if (dir) rmrfSync(dir)
  }
}

export const readText = (name: string) => fs.promises.readFile(name, 'utf8')
export const readJSON = async <T>(name: string) => JSON.parse(await readText(name)) as T

export const writeText = (name: string, data: string) => fs.promises.writeFile(name, data, 'utf8')
export const writeJSON = (name: string, data: object) => writeText(name, JSON.stringify(data, undefined, 2))

export const getFileSize = (filePath: string) => fs.promises.stat(filePath).then(stats => stats.size)

export async function moveFile(file: string, dir: string): Promise<string> {
  const newFile = path.join(dir, path.basename(file))

  await fs.promises.rename(file, newFile)

  return newFile
}
