import { noop } from '@v/utils'
import {
  test,
  assertEqual,
  assertTrue,
} from '@v/tester'
import { Observable } from './observable'
import {
  assertObservable,
  complete,
  error,
} from './test-utils'

test('observable completes', async () => {
  { // complete
    const o$ = Observable.from(1)

    await assertObservable(o$, [1, complete])
  }

  { // error
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.error('test')

      return noop
    })

    await assertObservable(o$, [1, error])
  }
})

test('destroy callback is executed', async () => {
  { // complete
    let destroyed = false
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.complete()

      return () => {
        destroyed = true
      }
    })

    await assertObservable(o$, [1, complete])
    assertTrue(destroyed)
  }

  { // error
    let destroyed = false
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.error('test')

      return () => {
        destroyed = true
      }
    })

    await assertObservable(o$, [1, error])
    assertTrue(destroyed)
  }
})

test('map', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.complete()

    return noop
  }).map(x => x + 1)

  await assertObservable(o$, [2, complete])
})

test('tap', async () => {
  let tap = 0
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.complete()

    return noop
  }).tap((x) => {
    tap = x
  })

  await assertObservable(o$, [1, complete])
  assertEqual(tap, 1)
})

test('filter', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.complete()

    return noop
  }).filter(x => x === 1)

  await assertObservable(o$, [1, complete])
})

test('switchMap', async () => {
  {
    let destCounter = 0

    const o$ = new Observable<number>((observer) => {
      observer.next(1)

      setTimeout(() => observer.next(2), 100)
      setTimeout(() => observer.complete(), 200)

      return () => {
        destCounter += 1
      }
    }).switchMap(() => new Observable((observer) => {
      observer.next(5)

      const timeout = setTimeout(() => {
        observer.next(1)
        observer.complete()
      }, 150)

      return () => {
        clearTimeout(timeout)
        destCounter += 1
      }
    }))

    await assertObservable(o$, [5, 5, 1, complete])

    assertEqual(destCounter, 3)
  }

  { // switchMap must wait for inner observable to complete
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.complete()

      return noop
    }).switchMap(() => new Observable<number>((observer) => {
      const timeout = setTimeout(() => {
        observer.next(2)
        observer.complete()
      }, 150)

      return () => {
        clearTimeout(timeout)
      }
    }))

    await assertObservable(o$, [2, complete])
  }
})

test('take', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.next(3)
    observer.complete()

    return noop
  }).take(2)

  await assertObservable(o$, [1, 2, complete])
})

test('buffer', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.next(3)
    observer.complete()

    return noop
  }).buffer(2)

  await assertObservable(o$, [[1], [1, 2], [2, 3], complete])
})

test('skip', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.next(3)
    observer.complete()

    return noop
  }).skip(2)

  await assertObservable(o$, [3, complete])
})

test('timeout', async () => {
  {
    const o$ = new Observable<number>((observer) => {
      const timeoutId = setTimeout(() => {
        observer.next(1)
        observer.complete()
      }, 100)

      return () => clearTimeout(timeoutId)
    }).timeout(10)

    await assertObservable(o$, [error])
  }

  {
    const o$ = new Observable<number>((observer) => {
      setTimeout(() => {
        observer.next(1)
      }, 50)

      setTimeout(() => {
        observer.complete()
      }, 200)

      return noop
    }).timeout(120)

    await assertObservable(o$, [1, complete])
  }
})
