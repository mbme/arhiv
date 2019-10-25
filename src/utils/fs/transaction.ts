import fs from 'fs'
import path from 'path'
import { createLogger } from '../logger'
import { AsyncCallbacks } from '../callbacks'
import { Counter } from '../counter'
import {
  createTempDir,
  fileExists,
  removeFile,
} from './utils'

const log = createLogger('fs-tx')

type FileData = string | Buffer

export class FSTransaction {
  private _revertCallbacks = new AsyncCallbacks()
  private _cleanupCallbacks = new AsyncCallbacks()
  private _tmpFileCounter = new Counter()
  private _completed = false

  private constructor(
    private _tmpDir: string,
  ) {
    const cleanup = async () => {
      try {
        await fs.promises.rmdir(_tmpDir)
        log.debug(`Removed temp dir ${_tmpDir}`)
      } catch (e) {
        log.warn(`failed to remove temp dir ${_tmpDir}: `, e)
      }
    }
    this._cleanupCallbacks.add(cleanup)
    this._revertCallbacks.add(cleanup)
  }

  static async create() {
    const tmpDir = await createTempDir()
    log.debug('Temp dir: ', tmpDir)

    return new FSTransaction(tmpDir)
  }

  private _assertNotCompleted() {
    if (this._completed) {
      throw new Error('already completed')
    }
  }

  async revert() {
    this._assertNotCompleted()
    this._completed = true

    await this._revertCallbacks.runAll(true, true)
    this._cleanupCallbacks.clear()
  }

  async complete() {
    this._assertNotCompleted()
    this._completed = true

    await this._cleanupCallbacks.runAll(true, true)
    this._revertCallbacks.clear()
  }

  async createFile(filePath: string, data: FileData) {
    this._assertNotCompleted()

    try {
      if (await fileExists(filePath)) {
        throw new Error('file already exists')
      }

      await fs.promises.writeFile(filePath, data)

      this._revertCallbacks.add(async () => {
        try {
          await removeFile(filePath)
          log.warn(`Reverted creating file ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo creating file ${filePath}: `, e)
        }
      })

    } catch (e) {
      log.error(`Failed to create file ${filePath}: `, e)

      throw e
    }
  }

  async updateFile(filePath: string, data: FileData) {
    this._assertNotCompleted()

    try {
      if (!await fileExists(filePath)) {
        throw new Error("file doesn't exist")
      }

      const _tmpFile = path.join(this._tmpDir, this._tmpFileCounter.incAndGet().toString())

      await fs.promises.rename(filePath, _tmpFile)

      this._cleanupCallbacks.add(async () => {
        try {
          await removeFile(_tmpFile)
        } catch (e) {
          log.warn(`Failed to remove backup file ${_tmpFile} after updating file ${filePath}: `, e)
        }
      })

      this._revertCallbacks.add(async () => {
        try {
          await fs.promises.rename(_tmpFile, filePath)
          log.warn(`Reverted updating file ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo updating file ${filePath}: `, e)
        }
      })

      await fs.promises.writeFile(filePath, data)
    } catch (e) {
      log.error(`Failed to update file ${filePath}: `, e)

      throw e
    }
  }

  async moveFile(filePath: string, newFilePath: string) {
    this._assertNotCompleted()

    try {
      if (!await fileExists(filePath)) {
        throw new Error("source file doesn't exist")
      }
      if (await fileExists(newFilePath)) {
        throw new Error('result file already exists')
      }

      await fs.promises.rename(filePath, newFilePath)

      this._revertCallbacks.add(async () => {
        try {
          await fs.promises.rename(newFilePath, filePath)
          log.warn(`Reverted moving file ${filePath} into ${newFilePath}`)
        } catch (e) {
          log.error(`Failed to undo moving file ${filePath} into ${newFilePath}: `, e)
        }
      })
    } catch (e) {
      log.error(`Failed to move file ${filePath} into ${newFilePath}: `, e)

      throw e
    }
  }

  async deleteFile(filePath: string) {
    this._assertNotCompleted()

    try {
      if (!await fileExists(filePath)) {
        throw new Error("file doesn't exist")
      }

      const _tmpFile = path.join(this._tmpDir, this._tmpFileCounter.incAndGet().toString())

      await fs.promises.rename(filePath, _tmpFile)

      this._cleanupCallbacks.add(async () => {
        try {
          await removeFile(_tmpFile)
        } catch (e) {
          log.warn(`Failed to remove backup file ${_tmpFile} after deleting file ${filePath}: `, e)
        }
      })

      this._revertCallbacks.add(async () => {
        try {
          await fs.promises.rename(_tmpFile, filePath)
          log.warn(`Reverted deleting file ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo deleting file ${filePath}: `, e)
        }
      })
    } catch (e) {
      log.error(`Failed to delete file ${filePath}: `, e)

      throw e
    }
  }

  async createDir(filePath: string) {
    this._assertNotCompleted()

    try {
      if (await fileExists(filePath)) {
        throw new Error('already exists')
      }

      await fs.promises.mkdir(filePath)

      this._revertCallbacks.add(async () => {
        try {
          await fs.promises.rmdir(filePath)
          log.warn(`Reverted creating directory ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo creating directory ${filePath}: `, e)
        }
      })

    } catch (e) {
      log.error(`Failed to create directory ${filePath}: `, e)

      throw e
    }
  }
}
