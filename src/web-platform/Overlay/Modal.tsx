import * as React from 'react'
import {
  stylish,
  theme,
} from '../style'
import { Overlay } from './Overlay'

const $modal = stylish({
  backgroundColor: theme.color.bg0,
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
      <div className={$modal.className}>
        {children}
      </div>
    </Overlay>
  )
}
