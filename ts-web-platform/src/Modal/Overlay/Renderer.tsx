import * as React from 'react'
import { TopOverlay } from './TopOverlay'
import {
  IOverlayRenderer,
  IOverlay,
  OverlayContext,
} from './context'

interface IProps {
  children: React.ReactNode,
}

function OverlayRenderer({ children }: IProps) {
  const [overlays, setOverlays] = React.useState<ReadonlyArray<[number, IOverlay]>>([])

  const renderer = React.useMemo<IOverlayRenderer>(() => ({
    show(id, overlay) {
      setOverlays(prevOverlays => [...prevOverlays, [id, overlay]])
    },
    hide(id) {
      setOverlays(prevOverlays => prevOverlays.filter(item => item[0] !== id))
    },
  }), [])

  const [id, topOverlay] = overlays[overlays.length - 1] || []

  return (
    <OverlayContext.Provider value={renderer}>
      {topOverlay && (
        <TopOverlay key={id} {...topOverlay} />
      )}
      {children}
    </OverlayContext.Provider>
  )
}

export default React.memo(OverlayRenderer)
