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
  zIndex: theme.zIndex.modal,

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

  onClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const {
      onClick,
    } = this.props

    if (onClick && e.target === e.currentTarget) {
      onClick()
    }
  }

  renderOverlay(content: React.ReactNode) {
    const {
      className,
    } = this.props

    this.context(
      content ? (
        <div
          className={classes(containerStyles, className)}
          onClick={this.onClick}
        >
          {content}
        </div>
      ) : null,
    )
  }

  componentDidMount() {
    this.renderOverlay(this.props.children)
  }

  componentDidUpdate() {
    this.renderOverlay(this.props.children)
  }

  componentWillUnmount() {
    this.renderOverlay(null)
  }

  render() {
    return null
  }
}
