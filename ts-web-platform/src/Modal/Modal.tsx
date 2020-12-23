import * as React from 'react'
import {
  useStyles,
  StyleArg,
} from '../core'
import Overlay from './Overlay'

const $modal: StyleArg = {
  backgroundColor: 'var(--color-bg0)',
  minWidth: '375px',
  border: 'default',
  boxShadow: 'default',
  height: 'auto',
}

interface IProps {
  children: React.ReactNode
  onCancel(): void
}

export const Modal = React.forwardRef(
  function Modal({ children, onCancel }: IProps, ref: React.Ref<HTMLDivElement>) {
    const className = useStyles($modal)

    return (
      <Overlay
        onClick={onCancel}
        innerRef={ref}
      >
        <div className={className}>
          {children}
        </div>
      </Overlay>
    )
  }
)
