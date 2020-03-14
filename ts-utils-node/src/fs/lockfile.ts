import { createLogger } from '@v/logger'
import {
  writeText,
  removeFile,
  fileExists,
} from './utils'

const log = createLogger('lockfile')

export class LockFile {
  constructor(
    private _lockFile: string,
  ) { }

  async create() {
    try {
      if (await fileExists(this._lockFile)) {
        throw new Error('already exist')
      }

      await writeText(this._lockFile, 'LockFile')
      log.debug(`created lockfile ${this._lockFile}`)
    } catch (e) {
      log.error(`failed to create lockfile ${this._lockFile}`)
      throw e
    }
  }

  async destroy() {
    await removeFile(this._lockFile)
  }
}
