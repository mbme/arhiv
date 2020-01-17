import { createLogger } from '~/logger'
import { Deferred } from './deferred'

const log = createLogger('queue')

type Action<T> = () => T
type Task<T> = [Deferred<T>, Action<T>]

export class Queue {
  private _queue: Task<any>[] = []

  private _taskId?: NodeJS.Immediate

  private _deferredClose?: Deferred<void>

  private _processQueue = async () => {
    while (this._queue.length) {
      const [deferred, action] = this._queue.shift()!

      try {
        const result = await Promise.resolve(action())
        deferred.resolve(result)
      } catch (e) {
        log.error('queued action failed', e)

        // tslint:disable-next-line:no-unsafe-any
        deferred.reject(e)
      }
    }

    this._taskId = undefined

    if (this._deferredClose) {
      this._deferredClose.resolve()
    }
  }

  private _scheduleQueueProcessing() {
    if (!this._taskId) {
      this._taskId = global.setImmediate(this._processQueue)
    }
  }

  isClosed() {
    return !!this._deferredClose
  }

  private _assertNotClosed() {
    if (this.isClosed()) {
      throw new Error('queue has been closed')
    }
  }

  async push<T>(action: Action<T>): Promise<T> {
    this._assertNotClosed()

    const deferred = new Deferred<T>()

    this._queue.push([deferred, action])
    this._scheduleQueueProcessing()

    return deferred.promise
  }

  async close() {
    this._assertNotClosed()

    this._deferredClose = new Deferred<void>()

    return this._deferredClose.promise
  }
}
