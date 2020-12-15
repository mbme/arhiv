// Pick all props from type T except enumerated in K
export type Without<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>

// Make props enumerated in TOptional optional
export type OptionalProps<T, TOptional extends keyof T> = (
  Without<T, TOptional> & Partial<Pick<T, TOptional>>
)

// Get type of object/class property
export type TypeOfProperty<T, P extends keyof T> = T[P]

export type Dict<T = string> = Record<string, T>

export type Obj = Dict<any>

export type EmptyObject = Dict<never>

// make all properties mutable
export type Mutable<T> = {
  -readonly [P in keyof T]: T[P];
}

export type Procedure = () => void
export type AsyncProcedure = () => Promise<void>

export type Constructor<T> = new (...args: any[]) => T

export type Result<T, E = Error> = { ok: true, value: T } | { ok: false, error: E }

export type ArrayElement<E> = E extends ReadonlyArray<infer T> ? T : never

export type ClassType<T> = new (...args: any[]) => T

export class ErrorResult<T> extends Error {
  constructor(public readonly result: T) {
    super()
  }
}
