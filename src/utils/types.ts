// Pick all props from type T except enumerated in K
export type Without<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>

// Make props enumerated in TOptional optional
export type OptionalProps<T, TOptional extends keyof T> = Without<T, TOptional> & Partial<Pick<T, TOptional>>

// Get type of object/class property
export type TypeOfProperty<T, P extends keyof T> = T[P]

export interface IDict<T = string> {
  [key: string]: T
}

export type Obj = IDict<any>

// make all properties mutable
export type Mutable<T> = {
  -readonly [P in keyof T]: T[P];
}

export type Procedure = () => void
export type AsyncProcedure = () => Promise<void>
