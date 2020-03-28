import * as React from 'react'
import {
  theme,
} from '../style'
import { Overlay } from './Overlay'
import { useStyles } from '@v/web-utils'

const $modal = {
  backgroundColor: theme.color.bg0,
  marginTop: '17vh',
  minWidth: '375px',
  padding: theme.spacing.medium,
  border: theme.border,
  boxShadow: theme.boxShadow,
}

interface IProps {
  children: React.ReactNode
  onCancel(): void
}

export function Modal({ children, onCancel }: IProps) {
  const className = useStyles($modal)

  return (
    <Overlay onClick={onCancel}>
      <div className={className}>
        {children}
      </div>
    </Overlay>
  )
}
