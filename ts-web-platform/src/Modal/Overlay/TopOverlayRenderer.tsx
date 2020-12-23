import * as React from 'react'
import { TopOverlay } from './TopOverlay'
import {
  OverlayRegistry,
} from './context'

export function TopOverlayRenderer() {
  const overlays = OverlayRegistry.useValues()

  const { id, item } = overlays[overlays.length - 1] || {}

  if (!item) {
    return null
  }

  return (
    <TopOverlay key={id} {...item} />
  )
}
