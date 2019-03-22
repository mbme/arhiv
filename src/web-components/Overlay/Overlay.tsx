import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'
import { Counter } from '~/utils'
import theme from '../theme'
import { OverlayContext } from './OverlayRenderer'

const containerStyles = style({
  backgroundColor: theme.color.backdrop,

  position: 'fixed',
  top: '0',
  right: '0',
  bottom: '0',
  left: '0',
  zIndex: theme.zIndex.modal,

  display: 'flex',
  justifyContent: 'center',
  alignItems: 'flex-start',
})

const idCounter = new Counter()

interface IProps {
  children: React.ReactNode
  onClick?(): void
  className?: string
}

export function Overlay({ children, onClick, className }: IProps) {
  const renderer = React.useContext(OverlayContext)

  const clickHandler = (e: React.MouseEvent<HTMLDivElement>) => {
    if (onClick && e.target === e.currentTarget) {
      onClick()
    }
  }

  React.useEffect(() => {
    const id = idCounter.incAndGet()
    renderer.show(id, (
      <div
        className={classes(containerStyles, className)}
        onClick={clickHandler}
      >
        {children}
      </div>
    ))

    return () => renderer.hide(id)
  })

  return null
}
