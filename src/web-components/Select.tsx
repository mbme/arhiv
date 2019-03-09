import React from 'react'
import {
  noop,
} from '~/utils'

interface IProps {
  options: { [key: string]: string }
  name: string
  onChange(value: string): void
}

export function Select({ options, onChange, ...other }: IProps) {
  const items = Object.entries(options).map(([key, label]) => (
    <option key={key} value={key}>
      {label}
    </option>
  ))

  return (
    <select
      onChange={e => onChange(e.target.value)}
      {...other}
    >
      {items}
    </select>
  )
}

export const examples = {
  '': (
    <Select
      name="select"
      options={{ val1: 'val1', val2: 'val2' }}
      onChange={noop}
    />
  ),
}
