import * as React from 'react'
import {
  style,
} from 'typestyle'
import theme from '../theme'
import { Overlay } from './Overlay'

const modalStyles = style({
  backgroundColor: theme.color.bg,
  marginTop: '17vh',
  minWidth: '375px',
  padding: theme.spacing.medium,
  border: theme.border,
  boxShadow: theme.boxShadow,
})

interface IProps {
  children: React.ReactNode
  onCancel(): void
}

export function Modal({ children, onCancel }: IProps) {
  return (
    <Overlay onClick={onCancel}>
      <div className={modalStyles}>
        {children}
      </div>
    </Overlay>
  )
}
