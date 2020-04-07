import * as React from 'react'
import {
  useCounter,
} from '@v/web-utils'
import { IOverlay, OverlayContext } from './context'

export function Overlay(props: IOverlay) {
  const renderer = OverlayContext.use()
  const id = useCounter()

  React.useEffect(() => {
    renderer.show(id, props)

    return () => renderer.hide(id)
  })

  return null
}
