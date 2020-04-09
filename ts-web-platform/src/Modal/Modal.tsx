import * as React from 'react'
import {
  useStyles,
  StyleArg,
} from '../core'
import { Overlay } from './Overlay'

const $modal: StyleArg = {
  backgroundColor: 'bg0',
  marginTop: '17vh',
  minWidth: '375px',
  padding: 'medium',
  border: 'default',
  boxShadow: 'default',
  height: 'auto',
}

interface IProps {
  children: React.ReactNode
  onCancel(): void
  innerRef?: React.RefObject<HTMLDivElement>
}

export function Modal({ children, onCancel, innerRef }: IProps) {
  const className = useStyles($modal)

  return (
    <Overlay onClick={onCancel} innerRef={innerRef}>
      <div className={className}>
        {children}
      </div>
    </Overlay>
  )
}
