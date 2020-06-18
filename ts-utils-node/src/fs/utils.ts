import fs from 'fs'
import os from 'os'
import path from 'path'
import {
  merge,
  prettyPrintJSON,
  consumeAsyncIterable,
  Obj,
} from '@v/utils'

interface IGetFilesOpts {
  skipDir?: string[]
  recursive?: boolean
  fileNames?: boolean // list file names or full paths
  dirs?: boolean // if should list dirs instead of files
}
const GET_FILES_OPTS = {
  skipDir: ['.git', 'node_modules'],
  recursive: true,
  fileNames: false,
  dirs: false,
}

// list files except skip dirs
export async function* getFiles(
  rootPath: string,
  options: IGetFilesOpts = GET_FILES_OPTS,
): AsyncIterableIterator<string> {
  const processedOpts = merge(GET_FILES_OPTS, options)

  for (const fileName of await fs.promises.readdir(rootPath)) {
    const filePath = path.resolve(rootPath, fileName)

    const isDir = await isDirectory(filePath)

    if (processedOpts.dirs === isDir) {
      yield processedOpts.fileNames ? fileName : filePath
      continue
    }

    if (!processedOpts.recursive) {
      continue
    }

    // traverse subdir only if it shouldn't be skipped
    if (processedOpts.skipDir!.includes(fileName)) {
      continue
    }

    yield* getFiles(filePath, options)
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

export const isFile = (filePath: string) => fs.promises.lstat(filePath).then(stats => stats.isFile())
export const isDirectory = (filePath: string) => fs.promises.lstat(filePath).then(stats => stats.isDirectory())

export async function listFiles(dir: string, opts: IGetFilesOpts = {}) {
  return consumeAsyncIterable(getFiles(dir, {
    recursive: false,
    fileNames: true,
    ...opts,
  }))
}

export async function listDirs(dir: string, opts: IGetFilesOpts = {}) {
  return consumeAsyncIterable(getFiles(dir, {
    recursive: false,
    fileNames: true,
    ...opts,
    dirs: true,
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
    if (dir) {
      rmrfSync(dir)
    }
  }

  return undefined
}

export const readText = (filePath: string) => fs.promises.readFile(filePath, 'utf8')
export const readJSON = async <T>(filePath: string) => JSON.parse(await readText(filePath)) as T

export const writeText = (filePath: string, data: string) => fs.promises.writeFile(filePath, data, 'utf8')
export const writeJSON = (filePath: string, data: Obj) => writeText(filePath, prettyPrintJSON(data))

export const getFileSize = (filePath: string) => fs.promises.stat(filePath).then(stats => stats.size)

// moving files (renaming file) across different file systems doesn't work
// so we need to 1) copy file into new destination 2) remove old file
async function moveFileAcrossDevices(filePath: string, newFilePath: string) {
  await fs.promises.copyFile(filePath, newFilePath)
  await removeFile(filePath)
}

export async function moveFile(filePath: string, newFilePath: string) {
  try {
    await fs.promises.rename(filePath, newFilePath)
  } catch (e) {
    if ((e as Obj).code === 'EXDEV') {
      await moveFileAcrossDevices(filePath, newFilePath)
    } else {
      throw e
    }
  }
}

export async function moveFileIntoDir(filePath: string, dir: string): Promise<string> {
  const newFile = path.join(dir, path.basename(filePath))

  await moveFile(filePath, newFile)

  return newFile
}

export async function removeFile(filePath: string, ignoreMissing = false) {
  if (ignoreMissing && !await fileExists(filePath)) {
    return undefined
  }

  return fs.promises.unlink(filePath)
}

export async function fileExists(filePath: string, assert = true) {
  const exists = await new Promise<boolean>((resolve) => {
    fs.access(filePath, fs.constants.F_OK, err => resolve(!err))
  })

  if (!exists) {
    return false
  }

  const _isFile = await isFile(filePath)
  if (assert && !_isFile) {
    throw new Error(`${filePath} exists but isn't a file`)
  }

  return _isFile
}

export async function dirExists(filePath: string, assert = true) {
  const exists = await new Promise<boolean>((resolve) => {
    fs.access(filePath, fs.constants.F_OK, err => resolve(!err))
  })

  if (!exists) {
    return false
  }

  const isDir = await isDirectory(filePath)
  if (assert && !isDir) {
    throw new Error(`${filePath} exists but isn't a directory`)
  }

  return isDir
}

export async function mkdir(filePath: string, recursive = false) {
  return fs.promises.mkdir(filePath, { recursive })
}

export async function assertDirExists(dir: string) {
  if (!await dirExists(dir, true)) {
    throw new Error(`Directory ${dir} doesn't exist`)
  }
}
