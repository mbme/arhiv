import * as React from 'react'
import {
  useStyles,
  StyleArg,
} from '@v/web-utils'
import {
  theme,
} from './style'

type ButtonVariant = 'primary' | 'secondary' | 'link'

interface IProps {
  onClick?(): void
  disabled?: boolean
  variant?: ButtonVariant
  children: React.ReactNode
  $style?: StyleArg
}

function getStyles(props: IProps) {
  return [
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

    props.disabled && {
      cursor: 'auto',
      color: theme.color.secondary,
      backgroundColor: theme.color.bg1,
    },

    !props.disabled && props.variant === 'primary' && {
      backgroundColor: theme.color.primary,
      color: theme.color.light,
      boxShadow: theme.boxShadow,
      '&:hover': {
        transform: 'scale(1.05)',
      },
    },

    !props.disabled && props.variant === 'secondary' && {
      color: theme.color.text,
      backgroundColor: theme.color.bg0,
      '&:hover': {
        backgroundColor: theme.color.bg1,
      },
    },

    !props.disabled && props.variant === 'link' && {
      border: '0 none',
      background: 'transparent',
      color: theme.color.link,
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
    >
      {children}
    </button>
  )
}
Button.defaultProps = {
  variant: 'secondary',
}
