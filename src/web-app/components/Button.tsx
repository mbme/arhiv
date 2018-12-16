import React from 'react'
import { classNames } from '../../utils'
import './Button.css'

interface IProps {
  onClick?: () => void
  disabled?: boolean
  primary?: boolean
  children: React.ReactNode
}
export default function Button({ onClick, disabled, primary, children }: IProps) {
  const className = classNames('Button', {
    'is-disabled': disabled,
    'is-primary': !disabled && primary,
    'is-secondary': !disabled && !primary,
  })

  return (
    <button
      className={className}
      onClick={onClick}
      disabled={disabled}
      type="button"
    >
      {children}
    </button>
  )
}
