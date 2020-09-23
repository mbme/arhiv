import { noop } from '@v/utils'
import * as React from 'react'
import {
  FocusManagerMode,
  FocusManager,
  FocusManagerContext,
} from './FocusContext'

function createFocusRegion(context: FocusManager) {
  interface IProps {
    children: React.ReactNode
  }

  return function FocusRegion({ children }: IProps) {
    return (
      <FocusManagerContext.Provider value={context}>
        {children}
      </FocusManagerContext.Provider>
    )
  }
}

export function useFocusRegion(mode: FocusManagerMode) {
  const [ref, setRef] = React.useState<HTMLElement | null>(null)
  const [context] = React.useState(() => new FocusManager(mode))

  React.useEffect(() => {
    if (!ref) {
      return noop
    }

    // FIXME maybe just install listeners?
    context.registerRegionNode(ref)

    return () => {
      context.unregisterRegionNode()
    }
  }, [ref])

  const parentContext = FocusManagerContext.use()

  // FIXME on mouse enter, is focused
  // FIXME register region ref, register in parentContext

  const FocusRegion = React.useMemo(() => createFocusRegion(context), [context])

  return {
    context,
    FocusRegion,
    setRef,
  }
}
