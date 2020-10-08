import * as React from 'react'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'

export function useFocusedRegion(): boolean {
  const context = React.useContext(FocusManagerContext)

  const [isEnabled, setIsEnabled] = React.useState(false)

  React.useEffect(() => {
    if (!context) {
      return noop
    }

    return context.enabled$.value$.subscribe({
      next: setIsEnabled,
    })
  }, [context])

  return isEnabled
}
