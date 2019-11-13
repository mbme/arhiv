import path from 'path'
import {
  consumeAsyncIterable,
  setLogLevel,
  createLogger,
} from '~/utils'
import { getFiles } from '~/utils/fs'
import { createRunnable } from '~/utils/runnable'
import { TestFile } from './test-file/test-file'

setLogLevel('ERROR')
const log = createLogger('tester')

createRunnable(async (...args: string[]) => {
  const filter = args.filter((arg) => !arg.startsWith('-'))[0] || ''
  const updateSnapshots = args.includes('-u')

  const basePath = path.join(process.env.BASE_DIR!, 'src')

  const files = (await consumeAsyncIterable(getFiles(basePath)))
    .filter((relPath) => (
      relPath.endsWith('.test.ts')
      && !relPath.includes('FLYCHECK')
      && relPath.includes(filter)
    ))

  const tests: TestFile[] = []
  let filesFailed = 0

  await Promise.all(files.map(async (file) => {
    try {
      const testFile = await TestFile.load(basePath, file, updateSnapshots)
      tests.push(testFile)
    } catch (e) {
      log.simple(`Failed to load test file ${file}: ${e}`)
      filesFailed += 1
    }
  }))
  log.simple(`${tests.length} matching test files`)
  log.simple('')

  let testsFailed = 0
  for (const test of tests) {
    try {
      testsFailed += await test.run()
    } catch (e) {
      log.simple(`Failed to run test file ${test.fileName}: ${e}`)
      filesFailed += 1
    }
  }

  if (filesFailed) {
    log.simple(`Test files failed: ${filesFailed}`)
  }

  log.simple(`Tests failed: ${testsFailed}`)

  if (!filesFailed && !testsFailed) {
    log.simple('Great Success!')
  }

  log.simple('')
})
