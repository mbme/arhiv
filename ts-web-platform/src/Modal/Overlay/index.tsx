import * as React from 'react'
import {
  useCounter,
} from '../../utils'
import { IOverlay, OverlayRegistry } from './context'

function Overlay(props: IOverlay) {
  const registry = OverlayRegistry.use()
  const id = useCounter()

  React.useEffect(() => {
    registry.put(id, props)

    return () => {
      registry.remove(id)
    }
  })

  return null
}

export default React.memo(Overlay)
