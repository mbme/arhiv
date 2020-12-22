import * as React from 'react'
import {
  StyleArg,
  useStyles,
} from './core'

type ButtonVariant = 'primary' | 'secondary' | 'link'

interface IProps {
  onClick?(): void
  disabled?: boolean
  variant?: ButtonVariant
  children: React.ReactNode
  $style?: StyleArg
}

function getStyles(props: IProps): StyleArg[] {
  return [
    {
      py: 'fine',
      px: 'medium',
      border: 'default',
      borderRadius: 'var(--border-radius-form)',
      cursor: 'pointer',
      userSelect: 'none',
      transition: 'background-color 100ms linear',

      textTransform: 'uppercase',
      letterSpacing: '1.2px',
      fontSize: '80%',
      whiteSpace: 'nowrap',
      overflow: 'hidden',
      textOverflow: 'ellipsis',
    },

    props.disabled && {
      cursor: 'auto',
      color: 'var(--color-secondary)',
      backgroundColor: 'var(--color-bg0)',
    },

    !props.disabled && props.variant === 'primary' && {
      backgroundColor: 'var(--color-primary)',
      color: 'var(--color-text-light)',
      boxShadow: 'default',
    },

    !props.disabled && props.variant === 'secondary' && {
      color: 'var(--color-text)',
      backgroundColor: 'var(--color-bg0)',
    },

    !props.disabled && props.variant === 'link' && {
      border: '0 none',
      background: 'transparent',
      color: 'var(--color-link)',
      px: '0',
      borderRadius: '0',
    },
  ]
}

export function Button(props: IProps) {
  const {
    onClick,
    disabled,
    children,
    $style,
  } = props

  const ref = React.useRef<HTMLButtonElement>(null)

  const className = useStyles(
    ...getStyles(props),
    $style,
  )

  return (
    <button
      className={className}
      onClick={onClick}
      disabled={disabled}
      type="button"
      ref={ref}
    >
      {children}
    </button>
  )
}
Button.defaultProps = {
  variant: 'secondary',
}
