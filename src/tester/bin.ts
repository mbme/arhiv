import path from 'path'
import {
  consumeAsyncIterable,
  setLogLevel,
  createLogger,
  termColors,
} from '~/utils'
import { getFiles } from '~/utils/fs'
import { createRunnable } from '~/utils/runnable'
import { TestFile } from './test-file/test-file'

setLogLevel('ERROR')
const log = createLogger('tester')

createRunnable(async (...args: string[]) => {
  const filter = args.filter((arg) => !arg.startsWith('-'))[0] || ''
  const updateSnapshots = args.includes('-u')

  const basePath = path.join(process.cwd(), process.env.BASE_DIR!)

  const files = (await consumeAsyncIterable(getFiles(basePath)))
    .filter((relPath) => relPath.endsWith('.test.js'))
  log.simple(`collected ${files.length} test files`)

  const tests: TestFile[] = []
  let filesFailed = 0

  for (const file of files) {
    if (!file.includes(filter)) {
      continue
    }

    try {
      const testFile = await TestFile.load(basePath, file, updateSnapshots)
      tests.push(testFile)
    } catch (e) {
      log.simple(`Failed to load test file ${file}: ${e}`)
      filesFailed += 1
    }
  }
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
    log.simple(termColors.red(`Test files failed: ${filesFailed}`))
  }

  if (testsFailed) {
    log.simple(termColors.red(`Tests failed: ${testsFailed}`))
  }

  if (!filesFailed && !testsFailed) {
    log.simple(termColors.green('Great Success!'))
  }

  log.simple('')
})
