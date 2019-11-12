import {
  test,
  assert,
} from '~/tester'
import { Observable } from './index'

const complete = Symbol('complete')
const error = Symbol('error')

async function assertObservable<T>(o$: Observable<T>, expected: Array<T | typeof complete | typeof error>) {
  const actual: typeof expected = []

  await new Promise((resolve) => {
    o$.subscribe({
      next(value) {
        actual.push(value)
      },
      error() {
        actual.push(error)
        resolve()
      },
      complete() {
        actual.push(complete)
        resolve()
      },
    })
  })

  assert.deepEqual(actual, expected)
}

test('observable completes', async () => {
  { // complete
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.complete()
    })

    await assertObservable(o$, [1, complete])
  }

  { // error
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.error('test')
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
    assert.true(destroyed)
  }

  { // error
    let destroyed = false
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.error('test')

      return () => destroyed = true
    })

    await assertObservable(o$, [1, error])
    assert.true(destroyed)
  }
})

test('map', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.complete()
  }).map(x => x + 1)

  await assertObservable(o$, [2, complete])
})

test('tap', async () => {
  let tap = 0
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.complete()
  }).tap(x => tap = x)

  await assertObservable(o$, [1, complete])
  assert.equal(tap, 1)
})

test('filter', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.complete()
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

      return () => destCounter += 1
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

    assert.equal(destCounter, 3)
  }

  { // switchMap must wait for inner observable to complete
    const o$ = new Observable<number>((observer) => {
      observer.next(1)
      observer.complete()
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
  }).take(2)

  await assertObservable(o$, [1, 2, complete])
})

test('buffer', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.next(3)
    observer.complete()
  }).buffer(2)

  await assertObservable(o$, [[1], [1, 2], [2, 3], complete])
})

test('skip', async () => {
  const o$ = new Observable<number>((observer) => {
    observer.next(1)
    observer.next(2)
    observer.next(3)
    observer.complete()
  }).skip(2)

  await assertObservable(o$, [3, complete])
})
