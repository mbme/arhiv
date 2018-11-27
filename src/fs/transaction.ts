import fs from 'fs'
import path from 'path'
import log from '../logger'
import { sha256 } from '../utils/node'
import { createTempDir, rmrfSync } from './utils'

type OperationType = 'ADD' | 'UPDATE' | 'REMOVE'
type FileData = string | Buffer

interface IOperation {
  type: OperationType
  filePath: string
  apply(tmpDir: string): Promise<void>
  rollback(): Promise<void>
}

const OPERATIONS = {
  ADD(filePath: string, data: FileData): IOperation {
    let _madeChanges = false

    return {
      type: 'ADD',
      filePath,

      async apply() {
        if (fs.existsSync(filePath)) throw new Error(`Can't ADD ${filePath}: already exists`)

        await fs.promises.writeFile(filePath, data)
        _madeChanges = true
      },

      async rollback() {
        if (_madeChanges) {
          await fs.promises.unlink(filePath)
        }
      },
    }
  },

  UPDATE(filePath: string, data: FileData): IOperation {
    let _tmpFile: string | undefined
    let _madeChanges = false

    return {
      type: 'UPDATE',
      filePath,

      async apply(tmpDir: string) {
        if (!fs.existsSync(filePath)) throw new Error(`Can't UPDATE ${filePath}: doesn't exist`)

        _tmpFile = path.join(tmpDir, sha256(filePath))
        await fs.promises.rename(filePath, _tmpFile)
        _madeChanges = true

        return fs.promises.writeFile(filePath, data)
      },

      async rollback() {
        if (_madeChanges) {
          await fs.promises.rename(_tmpFile!, filePath)
        }
      },
    }
  },

  REMOVE(filePath: string): IOperation {
    let _tmpFile: string | undefined
    let _madeChanges = false

    return {
      type: 'REMOVE',
      filePath,

      async apply(tmpDir: string) {
        if (!fs.existsSync(filePath)) throw new Error(`Can't REMOVE ${filePath}: doesn't exist`)

        _tmpFile = path.join(tmpDir, sha256(filePath))

        await fs.promises.rename(filePath, _tmpFile)
        _madeChanges = true
      },

      async rollback() {
        if (_madeChanges) {
          await fs.promises.rename(_tmpFile!, filePath)
        }
      },
    }
  },
}

export default function createFsTransaction() {
  const _operations: IOperation[] = []
  let _opCounter = 0

  function _assertUniqFile(filePath: string) {
    if (_operations.find((item) => item.filePath === filePath)) {
      throw new Error(`Operation with ${filePath} has already been scheduled`)
    }
  }

  async function _rollback() {
    log.warn(`Starting rollback of ${_opCounter} operations`)

    for (let i = 0; i < _opCounter; i += 1) {
      const operation = _operations[i]

      try {
        await operation.rollback()
      } catch (err) {
        if (err) {
          log.warn(`Rollback of ${operation.type} ${operation.filePath} failed:`, err)
        } else {
          log.warn(`Rollback of ${operation.type} ${operation.filePath} succeded`)
        }
      }
    }

    log.warn('Finished rollback')
  }

  return {
    addFile(filePath: string, data: FileData) {
      _assertUniqFile(filePath)
      _operations.push(OPERATIONS.ADD(filePath, data))
    },

    updateFile(filePath: string, data: FileData) {
      _assertUniqFile(filePath)
      _operations.push(OPERATIONS.UPDATE(filePath, data))
    },

    removeFile(filePath: string) {
      _assertUniqFile(filePath)
      _operations.push(OPERATIONS.REMOVE(filePath))
    },

    async commit() {
      log.debug(`Commiting ${_operations.length} operations`)

      let tmpDir
      try {
        tmpDir = await createTempDir()
        log.debug('Temp dir: ', tmpDir)

        for (const operation of _operations) {
          _opCounter += 1
          await operation.apply(tmpDir)
        }
      } catch (err) {
        await _rollback()
        throw err
      } finally {
        if (tmpDir) rmrfSync(tmpDir)
      }
    },
  }
}
