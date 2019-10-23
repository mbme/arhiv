import fs from 'fs'
import path from 'path'
import {
  createTempDir,
  fileExists,
  removeFile,
} from './fs'
import { AsyncCallbacks } from './callbacks'
import { createLogger } from './logger'
import { Counter } from './counter'

const log = createLogger('undoable-fs')

type FileData = string | Buffer

export class UndoableFS {
  private _undoCallbacks = new AsyncCallbacks()
  private _cleanupCallbacks = new AsyncCallbacks()
  private _tmpFileCounter = new Counter()
  private _undone = false

  private constructor(
    private _tmpDir: string,
  ) { }

  static async create() {
    const tmpDir = await createTempDir()
    log.debug('Temp dir: ', tmpDir)

    return new UndoableFS(tmpDir)
  }

  private _assertNotUndone() {
    if (this._undone) {
      throw new Error('already undone')
    }
  }

  async undo() {
    this._assertNotUndone()
    this._undone = true

    await this._undoCallbacks.runAll(true, true)
    this._cleanupCallbacks.clear()
  }

  async cleanup() {
    this._assertNotUndone()
    this._undone = true

    await this._cleanupCallbacks.runAll(true)
    this._undoCallbacks.clear()
  }

  async createFile(filePath: string, data: FileData) {
    this._assertNotUndone()

    try {
      if (await fileExists(filePath)) {
        throw new Error('file already exists')
      }

      await fs.promises.writeFile(filePath, data)

      this._undoCallbacks.add(async () => {
        try {
          await removeFile(filePath)
          log.warn(`Undone creating file ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo creating file ${filePath}: `, e)
        }
      })

    } catch (e) {
      log.error(`Failed to create file ${filePath}: `, e)
      await this.undo()
    }
  }

  async updateFile(filePath: string, data: FileData) {
    this._assertNotUndone()

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

      this._undoCallbacks.add(async () => {
        try {
          await fs.promises.rename(_tmpFile, filePath)
          log.warn(`Undone updating file ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo updating file ${filePath}: `, e)
        }
      })

      await fs.promises.writeFile(filePath, data)
    } catch (e) {
      log.error(`Failed to update file ${filePath}: `, e)
      await this.undo()
    }
  }

  async deleteFile(filePath: string) {
    this._assertNotUndone()

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

      this._undoCallbacks.add(async () => {
        try {
          await fs.promises.rename(_tmpFile, filePath)
          log.warn(`Undone deleting file ${filePath}`)
        } catch (e) {
          log.error(`Failed to undo deleting file ${filePath}: `, e)
        }
      })
    } catch (e) {
      log.error(`Failed to delete file ${filePath}: `, e)
      await this.undo()
    }
  }
}
