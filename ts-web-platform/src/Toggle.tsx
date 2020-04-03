/* eslint-disable max-len */
import * as React from 'react'
import { useCounter } from '@v/web-utils'
import {
  StyleArg,
  useStyles,
} from './core'

// based on Android style https://github.com/JNKKKK/MoreToggles.css
const colors = {
  bg: '#848484',
  bgChecked: '#7cbcbf',
  bgDisabled: '#ccc',
  circle: '#fff',
  circleDisabled: '#ccc',
}

const $wrapper: StyleArg = {
  display: 'inline-block',
  boxSizing: 'border-box',
  fontSize: '0.6rem',
}

const $label: StyleArg = {
  // background
  display: 'inline-block',
  position: 'relative',
  boxSizing: 'border-box',
  cursor: 'pointer',
  width: '4em',
  height: '1.75em',
  borderRadius: '0.875em',
  backgroundImage: `linear-gradient(to right, ${colors.bg} 0%, ${colors.bg} 50%, ${colors.bgChecked} 50%, ${colors.bgChecked} 100%)`,
  backgroundSize: '8em 1.7em',
  transition: 'all 0.3s ease',

  // circle
  '&:before': {
    content: '""',
    display: 'block',
    position: 'absolute',
    width: '2.25em',
    height: '2.25em',
    top: '-0.25em',
    left: '0',
    borderRadius: '2em',
    background: colors.circle,
    transition: '0.3s ease',
    boxShadow: '0 0.125em 0.375em rgba(0, 0, 0, 0.5)',
  },
}

const $input: StyleArg = {
  position: 'absolute',
  boxSizing: 'border-box',
  height: '0',
  width: '0',
  fontSize: 'inherit',
  margin: '0',
  border: 'none',
  zIndex: 1,
  cursor: 'pointer',
  '-moz-appearance': 'none',
  '-webkit-appearance': 'none',
  '&:focus': {
    outline: 'none',
  },

  ['&:checked+label' as any]: { // background
    backgroundPosition: '-100%',
  },
  ['&:checked+label:before' as any]: { // circle
    transform: 'translateX(1.75em)',
  },

  // disabled
  ['&:disabled+label' as any]: { // background
    background: `${colors.bgDisabled} !important`,
    cursor: 'not-allowed !important',
  },
  ['&:disabled+label:before' as any]: { // circle
    background: `${colors.circleDisabled} !important`,
    boxShadow: '0 0.125em 0.375em rgba(0, 0, 0, 0.5) !important',
  },
}

interface IProps extends Omit<React.HTMLProps<HTMLInputElement>, 'onChange' | 'className'> {
  onChange(checked: boolean): void
}

export function Toggle({ onChange, ...props }: IProps) {
  const id = useCounter()
  const wrapperClassName = useStyles($wrapper)
  const labelClassName = useStyles($label)
  const inputClassName = useStyles($input)

  return (
    <div className={wrapperClassName}>
      <input
        id={id.toString()}
        className={inputClassName}
        type="checkbox"
        onChange={e => onChange(e.target.checked)}
        {...props}
      />
      <label
        className={labelClassName}
        htmlFor={id.toString()}
      >
      </label>
    </div>
  )
}
