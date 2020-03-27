import * as React from 'react'

export function createContext<T>(initialValue?: T) {
  const Context = React.createContext<T | undefined>(initialValue)

  return {
    Provider: Context.Provider,

    use(): T {
      const value = React.useContext(Context)

      if (!value) {
        throw new Error("Context doesn't have a value yet")
      }

      return value
    },
  }
}
