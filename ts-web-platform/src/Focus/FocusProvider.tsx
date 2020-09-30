import { createLogger } from '@v/logger'
import { getLastEl, removeMut } from '@v/utils'
import * as React from 'react'
import { FocusManager } from './focus-manager'

const log = createLogger('FocusStack')

class FocusStack {
  private _stack: FocusManager[] = []

  add(focusManager: FocusManager) {
    for (const rootFocusManager of this._stack) {
      rootFocusManager.disable()
    }

    this._stack.push(focusManager)
    log.debug(`add ${focusManager.name}`)

    focusManager.enable()

    return () => {
      removeMut(this._stack, focusManager)
      log.debug(`remove ${focusManager.name}`)

      getLastEl(this._stack)?.enable()
    }
  }
}

export const FocusStackContext = React.createContext<FocusStack | undefined>(undefined)

interface IProps {
  children: React.ReactNode
}

export function FocusProvider({ children }: IProps) {
  const [stack] = React.useState(() => new FocusStack())

  return (
    <FocusStackContext.Provider value={stack}>
      {children}
    </FocusStackContext.Provider>
  )
}
