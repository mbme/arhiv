import {
  TestContext,
  initializeTestContext,
  getTestContext,
} from './test-context'

export class TestFile {
  private constructor(
    private _testContext: TestContext,
  ) { }

  static async load(testFile: string) {
    initializeTestContext()

    require(testFile) // collect tests from the file into test context

    return new TestFile(getTestContext())
  }
}
