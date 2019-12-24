/* tslint:disable:no-console */
import path from 'path'
import {
  consumeAsyncIterable,
  termColors,
} from '~/utils'
import {
  configureLogger,
} from '~/logger'
import {
  getFiles,
} from '~/utils/fs'
import {
  App,
  command,
} from '~/utils/app'
import {
  TestFile,
} from './test-file/test-file'

configureLogger({
  minLogLevel: 'ERROR', // suppress log messages
})

App.create('tester')
  .addCommand(
    command('', '')
      .option('-u', 'update changed snapshots')
      .positional('filter', 'filter to apply to test files'),
    async (options) => {
      const updateSnapshots = options['-u'] !== undefined

      const basePath = path.join(process.cwd(), process.env.BASE_DIR!)
      const srcPath = path.join(process.cwd(), process.env.SRC_DIR!)

      const files = (await consumeAsyncIterable(getFiles(basePath)))
        .filter((relPath) => relPath.endsWith('.test.js'))
        .filter((relPath) => !relPath.includes('FLYCHECK'))

      console.log(`collected ${files.length} test files`)

      const tests: TestFile[] = []
      let filesFailed = 0

      for (const file of files) {
        if (options.filter && !file.includes(options.filter)) {
          continue
        }

        try {
          const testFile = await TestFile.load(basePath, srcPath, file, updateSnapshots)
          tests.push(testFile)
        } catch (e) {
          console.log(`Failed to load test file ${file}: ${e}`)
          filesFailed += 1
        }
      }
      console.log(`${tests.length} matching test files`)
      console.log('')

      let testsFailed = 0
      for (const test of tests) {
        try {
          testsFailed += await test.run()
        } catch (e) {
          console.log(`Failed to run test file ${test.fileName}: ${e}`)
          filesFailed += 1
        }
      }

      if (filesFailed) {
        console.log(termColors.red(`Test files failed: ${filesFailed}`))
      }

      if (testsFailed) {
        console.log(termColors.red(`Tests failed: ${testsFailed}`))
      }

      if (!filesFailed && !testsFailed) {
        console.log(termColors.green('Great Success!'))
      }

      console.log('')
    },
  ).run()
