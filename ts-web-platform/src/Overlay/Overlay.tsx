import * as React from 'react'
import { noop } from '@v/utils'
import {
  createContext,
  useCounter,
} from '@v/web-utils'
import {
  useStyles,
  StyleArg,
} from '../core'

const $container: StyleArg = {
  backgroundColor: 'bgOverlay',

  position: 'fixed',
  top: '0',
  right: '0',
  bottom: '0',
  left: '0',
  zIndex: 'modal',

  display: 'flex',
  justifyContent: 'center',
  alignItems: 'flex-start',
}

interface IOverlay {
  children: React.ReactNode
  onClick?(): void
  $styles?: StyleArg[]
}

interface IOverlayRenderer {
  show(id: number, overlay: IOverlay): void
  hide(id: number): void
}

const OverlayContext = createContext<IOverlayRenderer>()

function TopOverlay({ children, onClick, $styles = [] }: IOverlay) {
  const className = useStyles($container, ...$styles)

  const clickHandler = (e: React.MouseEvent<HTMLDivElement>) => {
    if (onClick && e.target === e.currentTarget) {
      onClick()
    }
  }

  React.useEffect(() => {
    if (!onClick) {
      return noop
    }

    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClick()
      }
    }

    document.addEventListener('keydown', handler)

    return () => document.removeEventListener('keydown', handler)
  }, [onClick])

  return (
    <div
      className={className}
      onClick={clickHandler}
      role="dialog"
      aria-modal="true"
    >
      {children}
    </div>
  )
}

export function Overlay(props: IOverlay) {
  const renderer = OverlayContext.use()
  const id = useCounter()

  React.useEffect(() => {
    renderer.show(id, props)

    return () => renderer.hide(id)
  }, [])

  return null
}

interface IProps {
  children: React.ReactNode,
}

export const OverlayRenderer = React.memo(({ children }: IProps) => {
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
})
