import * as React from 'react'
import {
  theme,
} from './style'

type ButtonVariant = 'primary' | 'secondary' | 'link'

const $button = stylish(
  {
    padding: `${theme.spacing.fine} ${theme.spacing.medium}`,
    border: theme.border,
    borderRadius: '2px',
    cursor: 'pointer',
    userSelect: 'none',
    transition: 'background-color 100ms linear, transform 50ms ease-in',

    textTransform: 'uppercase',
    letterSpacing: '1.2px',
    fontSize: '80%',
  },

  props => props.disabled && {
    cursor: 'auto',
    color: theme.color.secondary,
    backgroundColor: theme.color.bg1,
  },

  props => !props.disabled && props.variant === 'primary' && {
    backgroundColor: theme.color.primary,
    color: theme.color.light,
    boxShadow: theme.boxShadow,
    '&:hover': {
      transform: 'scale(1.05)',
    },
  },

  props => !props.disabled && props.variant === 'secondary' && {
    color: theme.color.text,
    backgroundColor: theme.color.bg0,
    '&:hover': {
      backgroundColor: theme.color.bg1,
    },
  },

  props => !props.disabled && props.variant === 'link' && {
    border: '0 none',
    background: 'transparent',
    color: theme.color.link,
  },
)

interface IProps {
  onClick?(): void
  disabled?: boolean
  variant?: ButtonVariant
  children: React.ReactNode
  $style?: $Style
}

export function Button(props: IProps) {
  const {
    onClick,
    disabled,
    children,
    $style,
  } = props

  return (
    <button
      className={$button.and($style).with(props).className}
      onClick={onClick}
      disabled={disabled}
      type="button"
    >
      {children}
    </button>
  )
}
Button.defaultProps = {
  variant: 'secondary',
}
