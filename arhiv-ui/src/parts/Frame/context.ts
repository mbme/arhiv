import * as React from 'react'
import { withoutUndefined } from '@v/utils'
import { createRegistry, useCounter } from '@v/web-platform'

export interface IAction {
  onClick(): void,
  disabled?: boolean,
  children: React.ReactNode,
}

export const ActionsRegistry = createRegistry<IAction[]>()

export function useActions(getActions: () => Array<IAction | undefined>, deps: any[] = []) {
  const registry = ActionsRegistry.use()
  const id = useCounter()

  React.useEffect(() => {
    const newActions = withoutUndefined(getActions())

    registry.put(id, newActions)

    return () => {
      registry.remove(id)
    }
  }, deps)
}
