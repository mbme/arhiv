import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'
import theme from '../theme'
import { OverlayContext } from './OverlayRenderer'

const containerStyles = style({
  backgroundColor: theme.color.backdrop,

  position: 'fixed',
  top: '0',
  right: '0',
  bottom: '0',
  left: '0',
  zIndex: 10,

  display: 'flex',
  justifyContent: 'center',
  alignItems: 'flex-start',
})

interface IProps {
  children: React.ReactNode
  onClick?(): void
  className?: string
}
export class Overlay extends React.PureComponent<IProps> {
  static contextType = OverlayContext
  context!: React.ContextType<typeof OverlayContext>

  rootEl = document.getElementById('modal')!

  componentDidMount() {
    this.renderOverlay()
  }

  componentDidUpdate() {
    this.renderOverlay()
  }

  onClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const {
      onClick,
    } = this.props

    if (onClick && e.target === e.currentTarget) {
      onClick()
    }
  }

  renderOverlay() {
    const {
      className,
      children,
    } = this.props

    this.context(
      <div
        className={classes(containerStyles, className)}
        onClick={this.onClick}
      >
        {children}
      </div>,
    )
  }

  render() {
    return null
  }
}
