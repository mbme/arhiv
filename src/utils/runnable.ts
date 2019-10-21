export function createRunnable(run: (...args: string[]) => Promise<void> | void) {
  const args = process.argv.slice(3)

  Promise.resolve(run(...args)).catch((e) => {
    // tslint:disable-next-line:no-console
    console.error('process failed', e)

    process.exit(2)
  })
}
