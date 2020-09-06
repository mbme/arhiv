import * as React from 'react'
import { useFormControl } from './Form'

interface IProps extends Omit<React.HTMLProps<HTMLSelectElement>, 'onChange' | 'value'> {
  name: string
  options: { [key: string]: string }
}

export function Select({ options, name, ...other }: IProps) {
  const {
    value,
    setValue,
  } = useFormControl(name)

  const items = Object.entries(options).map(([key, label]) => (
    <option key={key} value={key}>
      {label}
    </option>
  ))

  return (
    <select
      name={name}
      value={value}
      onChange={e => setValue(e.target.value)}
      {...other}
    >
      {items}
    </select>
  )
}
