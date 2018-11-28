import log from '../logger'

const scheduleTask = global.setImmediate || ((task) => setTimeout(task, 0))

type OnClose = () => void
type Task = () => Promise<any>

export default function createQueue() {
  let _taskId: NodeJS.Immediate | undefined
  let _onClose: OnClose | undefined
  const _queue: Task[] = []

  async function processQueue() {
    while (_queue.length) {
      const action = _queue.shift()!
      await action().catch((e) => log.error('queued action failed', e))
    }
    _taskId = undefined
    if (_onClose) _onClose()
  }

  const scheduleQueueProcessing = () => {
    if (!_taskId) {
      _taskId = scheduleTask(processQueue)
    }
  }

  return {
    push<T>(action: () => Promise<T>): Promise<T> {
      return new Promise((resolve, reject) => {
        if (_onClose) throw new Error('queue has been closed')

        _queue.push(() => action().then(resolve, reject))
        scheduleQueueProcessing()
      })
    },

    close() {
      return new Promise((resolve) => {
        _onClose = resolve
        scheduleQueueProcessing()
      })
    },
  }
}
