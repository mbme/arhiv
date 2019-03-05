import React, { PureComponent } from 'react'
import ReactDOM from 'react-dom'
import { classNames } from '../../utils'
import './Backdrop.css'

export type BackdropClickEvent = React.MouseEvent<HTMLDivElement>

interface IProps {
  children: React.ReactNode
  onClick?(e: BackdropClickEvent): void
  className?: string
}
export default class Backdrop extends PureComponent<IProps> {
  rootEl = document.getElementById('modal')!

  render() {
    const {
      className,
      onClick,
      children,
    } = this.props

    return ReactDOM.createPortal(
      <div className={classNames('Backdrop-container', className)} onClick={onClick}>
        {children}
      </div>,
      this.rootEl,
    )
  }
}
