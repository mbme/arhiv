import * as React from 'react'
import {
  OverlayRegistry,
} from './context'
import { TopOverlayRenderer } from './TopOverlayRenderer'

interface IProps {
  children: React.ReactNode,
}

function OverlayRenderer({ children }: IProps) {
  return (
    <OverlayRegistry.Provider>
      <TopOverlayRenderer />

      {children}
    </OverlayRegistry.Provider>
  )
}

export default React.memo(OverlayRenderer)
