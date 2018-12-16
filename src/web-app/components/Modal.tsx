import React, { PureComponent } from 'react'
import Button from './Button'
import Backdrop, { BackdropClickEvent } from './Backdrop'
import './Modal.css'

interface IModalProps {
  children: React.ReactNode
  onCancel: () => void
}
export default class Modal extends PureComponent<IModalProps, {}> {
  onModalClick = (e: BackdropClickEvent) => {
    if (e.target === e.currentTarget) this.props.onCancel()
  }

  render() {
    return (
      <Backdrop onClick={this.onModalClick}>
        <div className="Modal-modal">{this.props.children}</div>
      </Backdrop>
    )
  }
}

interface IDialogProps {
  children: React.ReactNode
  confirmation: React.ReactNode
  onConfirmed: () => void
  onCancel: () => void
}
export function ConfirmationDialog({ children, confirmation, onConfirmed, onCancel }: IDialogProps) {
  return (
    <Modal onCancel={onCancel}>
      <div className="g-section">
        {children}
      </div>
      <div className="Modal-buttons">
        <Button onClick={onCancel}>CANCEL</Button>
        <Button primary onClick={onConfirmed}>{confirmation}</Button>
      </div>
    </Modal>
  )
}
