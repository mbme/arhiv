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
  innerRef?: React.Ref<HTMLButtonElement>
}

function getStyles(props: IProps): StyleArg[] {
  return [
    {
      py: 'fine',
      px: 'medium',
      border: 'default',
      borderRadius: '2px',
      cursor: 'pointer',
      userSelect: 'none',
      transition: 'background-color 100ms linear, transform 50ms ease-in',

      textTransform: 'uppercase',
      letterSpacing: '1.2px',
      fontSize: '80%',
      whiteSpace: 'nowrap',
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
      '&:hover': {
        transform: 'scale(1.05)',
      },
    },

    !props.disabled && props.variant === 'secondary' && {
      color: 'var(--color-text)',
      backgroundColor: 'var(--color-bg0)',
      '&:hover': {
        transform: 'scale(1.05)',
      },
    },

    !props.disabled && props.variant === 'link' && {
      border: '0 none',
      background: 'transparent',
      color: 'var(--color-link)',
    },
  ]
}

export function Button(props: IProps) {
  const {
    onClick,
    disabled,
    children,
    $style,
    innerRef,
  } = props

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
      ref={innerRef}
    >
      {children}
    </button>
  )
}
Button.defaultProps = {
  variant: 'secondary',
}
