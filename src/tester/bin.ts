import path from 'path'
import { consumeAsyncIterable } from '~/utils'
import { getFiles } from '~/utils/fs'
import log, { setLogLevel } from '~/logger'
import {
  getTestPlan,
  initTestPlan,
  runTests,
} from './index'

setLogLevel('ERROR')

export default async function run(...args: string[]) {
  const filter = args.filter((arg) => !arg.startsWith('-'))[0] || ''
  const updateSnapshots = args.includes('-u')

  const basePath = path.join(process.env.BASE_DIR!, 'src')

  const testFiles = (await consumeAsyncIterable(getFiles(basePath)))
    .filter((relPath) => (
      relPath.endsWith('.test.ts')
      && !relPath.includes('FLYCHECK')
      && relPath.includes(filter)
    ))

  const testPlans = []
  for (const testFile of testFiles) {
    initTestPlan()
    require(testFile)
    const testPlan = getTestPlan()

    const only = testPlan.tests.find((test) => test.only)
    if (only) {
      if (updateSnapshots) throw new Error("Can't update the 'only' snapshot")

      testPlans.length = 0
      testPlans.push({ file: testFile, ...testPlan, tests: [only] })
      break
    }

    testPlans.push({ file: testFile, ...testPlan })
  }

  let failures = 0

  for (const testPlan of testPlans) {
    log.simple(path.relative(basePath, testPlan.file))

    if (testPlan.before) {
      await Promise.resolve(testPlan.before())
    }

    failures += await runTests(
      testPlan.file,
      testPlan.tests,
      updateSnapshots,
    )

    if (testPlan.after) {
      await Promise.resolve(testPlan.after())
    }

    log.simple('')
  }

  log.simple(failures ? `Failures: ${failures}` : 'Success!', '\n')
}
