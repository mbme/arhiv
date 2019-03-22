// Pick all props from type T except enumerated in K
export type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>

// Make props enumerated in TOptional optional
export type OptionalProps<T, TOptional extends keyof T> = Omit<T, TOptional> & Partial<Pick<T, TOptional>>

// Get type of object/class property
export type TypeOfProperty<T, P extends keyof T> = T[P]
