export interface ILazy<T> {
  readonly initialized: boolean
  readonly value: T
}

export function lazy<T>(createVal: () => T): ILazy<T> {
  let val: T
  let initialized = false

  return {
    get initialized() {
      return initialized
    },
    get value(): T {
      if (!val) {
        initialized = true
        val = createVal()
      }

      return val
    },
  }
}
