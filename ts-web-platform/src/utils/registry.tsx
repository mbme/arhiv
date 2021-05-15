import * as React from 'react'
import { Cell } from '@v/reactive'
import { replaceAtMut } from '@v/utils'
import { useCell } from './hooks'

interface IRegistryItem<T> {
  id: number
  item: T
}

type RegistryState<T> = Array<IRegistryItem<T>>

export class Registry<T> {
  readonly cell$ = new Cell<RegistryState<T>>([])

  put(id: number, item: T) {
    const prevValues = this.cell$.value

    const pos = prevValues.findIndex(item => item.id === id)

    if (pos === -1) {
      this.cell$.value = [ ...prevValues, { id, item } ]
      return
    }

    this.cell$.value = replaceAtMut([...prevValues], pos, { id, item })
  }

  remove(id: number) {
    const prevValues = this.cell$.value

    this.cell$.value = prevValues.filter(item => item.id !== id)
  }
}

export function createRegistry<T>() {
  const context = React.createContext<Registry<T> | undefined>(undefined)

  function Provider({ children }: { children: React.ReactNode }) {
    const [registry] = React.useState(() => new Registry<T>())

    return (
      <context.Provider value={registry}>
        {children}
      </context.Provider>
    )
  }

  return {
    use(): Registry<T> {
      const registry = React.useContext(context)
      if (!registry) {
        throw new Error('Registry not provided')
      }

      return registry
    },

    useValues(): IRegistryItem<T>[] {
      const registry = React.useContext(context)
      if (!registry) {
        throw new Error('Registry not provided')
      }

      const [values] = useCell(registry.cell$)

      return values
    },

    Provider,
  }
}
