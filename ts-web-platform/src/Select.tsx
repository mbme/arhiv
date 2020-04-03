import * as React from 'react'

interface IProps extends Omit<React.HTMLProps<HTMLSelectElement>, 'onChange'> {
  options: { [key: string]: string }
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
