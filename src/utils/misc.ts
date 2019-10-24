export async function promiseTimeout(timeout: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, timeout))
}

// tslint:disable-next-line:no-empty
export const noop = () => { }

export const identity = <T>(x: T) => x

export async function consumeAsyncIterable<T>(iterable: AsyncIterableIterator<T>): Promise<T[]> {
  const result: T[] = []
  for await (const item of iterable) {
    result.push(item)
  }

  return result
}

export const prettyPrintJSON = (data: any) => JSON.stringify(data, undefined, 2)
