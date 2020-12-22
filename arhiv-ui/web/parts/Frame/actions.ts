import { withoutUndefined } from '@v/utils'
import * as React from 'react'

export interface IAction {
  onClick(): void,
  disabled?: boolean,
  children: React.ReactNode,
}

export const ActionsContext = React.createContext<(currentActions: IAction[]) => IAction[]>(() => [])

export function useActions(getActions: () => Array<IAction | undefined>, deps: any[] = []) {
  const setActions = React.useContext(ActionsContext)

  React.useEffect(() => {
    const newActions = withoutUndefined(getActions())
    setActions(newActions)

    return () => {

    }
  }, deps)
}
