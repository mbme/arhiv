/* eslint-disable no-console */
import {
  consumeAsyncIterable,
  termColors,
} from '@v/utils'
import {
  configureLogger,
} from '@v/logger'
import {
  fs,
  cli,
} from '@v/utils-node'
import {
  TestFile,
} from './test-file/test-file'
import packageJson from '../../package.json'

configureLogger({
  minLogLevel: 'ERROR', // suppress log messages
})

async function listFiles(srcPath: string): Promise<string[]> {
  const options = {
    skipDir: ['.git', 'node_modules', 'target'],
  }

  const wsDirs = packageJson.workspaces.map(wsDir => `${srcPath}/${wsDir}`)

  return (await Promise.all(wsDirs.map(wsDir => consumeAsyncIterable(fs.getFiles(wsDir, options))))).flat()
}

cli.CliApp.create('tester')
  .addCommand(
    cli.command('', 'Run all tests')
      .option('-u', 'update changed snapshots')
      .positional('filter', 'filter to apply to test files'),
    async (options) => {
      const updateSnapshots = options['-u'] !== undefined

      const srcPath = process.cwd()

      const files = (await listFiles(srcPath))
        .filter(relPath => relPath.endsWith('.test.ts'))
        .filter(relPath => !relPath.includes('FLYCHECK'))

      console.log(`collected ${files.length} test files`)

      const tests: TestFile[] = []
      let filesFailed = 0

      for (const file of files) {
        if (options.filter && !file.includes(options.filter)) {
          continue
        }

        try {
          const testFile = TestFile.load(srcPath, file, updateSnapshots)
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
