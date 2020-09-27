import * as React from 'react'
import { useCell } from '@v/web-utils'
import { FocusManagerContext } from './context'

export function useFocusedRegion(): boolean {
  const context = React.useContext(FocusManagerContext)
  if (!context) {
    throw new Error('FocusManager must be provided')
  }

  const [isEnabled] = useCell(context.enabled$)

  return isEnabled
}
