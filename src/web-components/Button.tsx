import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'
import theme from './theme'

const baseStyle = style({
  padding: `${theme.spacing.fine} ${theme.spacing.medium}`,
  border: theme.border,
  borderRadius: '2px',
  cursor: 'pointer',
  userSelect: 'none',
  transition: 'background-color 100ms linear, transform 50ms ease-in',

  textTransform: 'uppercase',
  letterSpacing: '1.2px',
  fontSize: '80%',
})

const disabledStyle = style({
  cursor: 'auto',
  color: theme.color.secondary,
  backgroundColor: theme.color.bgDarker,
})

const primaryStyle = style({
  backgroundColor: theme.color.primary,
  color: theme.color.light,
  boxShadow: theme.boxShadow,
  $nest: {
    '&:hover': {
      transform: 'scale(1.05)',
    },
  },
})

const secondaryStyle = style({
  color: theme.color.text,
  backgroundColor: theme.color.bg,
  $nest: {
    '&:hover': {
      backgroundColor: theme.color.bgDarker,
    },
  },
})

interface IProps {
  className?: string
  onClick?(): void
  disabled?: boolean
  primary?: boolean
  children: React.ReactNode
}

export function Button({ className, onClick, disabled, primary, children }: IProps) {
  const styles = classes(
    className,
    baseStyle,
    disabled && disabledStyle,
    !disabled && primary && primaryStyle,
    !disabled && !primary && secondaryStyle,
  )

  return (
    <button
      className={styles}
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
