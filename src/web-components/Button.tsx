import * as React from 'react'
import { Stylish, stylish } from '~/stylish'
import theme from './theme'

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
    backgroundColor: theme.color.bgDarker,
  },

  props => !props.disabled && props.primary && {
    backgroundColor: theme.color.primary,
    color: theme.color.light,
    boxShadow: theme.boxShadow,
    '&:hover': {
      transform: 'scale(1.05)',
    },
  },

  props => !props.disabled && !props.primary && {
    color: theme.color.text,
    backgroundColor: theme.color.bg,
    '&:hover': {
      backgroundColor: theme.color.bgDarker,
    },
  },
)

interface IProps {
  onClick?(): void
  disabled?: boolean
  primary?: boolean
  children: React.ReactNode
  $style?: Stylish
}

export function Button(props: IProps) {
  const {
    onClick,
    disabled,
    children,
  } = props

  return (
    <button
      className={$button.with(props).className}
      onClick={onClick}
      disabled={disabled}
      type="button"
    >
      {children}
    </button>
  )
}

export const examples = {
  'Primary': (
    <Button primary>Primary Button</Button>
  ),
  'Primary disabled': (
    <Button primary disabled>Primary Button</Button>
  ),

  'Secondary': (
    <Button>Button</Button>
  ),
  'Secondary disabled': (
    <Button disabled>Button</Button>
  ),
}
