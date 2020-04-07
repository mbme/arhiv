import * as React from 'react'
import { noop } from '@v/utils'
import {
  useStyles,
  StyleArg,
} from '../../core'
import { IOverlay } from './context'

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

export function TopOverlay({ children, onClick, $styles = [] }: IOverlay) {
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
