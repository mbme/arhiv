// tslint:disable:react-hooks-nesting

import * as React from 'react'
import { Input } from './Input'

export const examples = {
  'Light input': () => {
    const [value, setValue] = React.useState('')

    return (
      <Input
        name="input1"
        placeholder="Input example (light)"
        value={value}
        light
        onChange={setValue}
      />
    )
  },

  'Light input with clear': () => {
    const [value, setValue] = React.useState('')

    return (
      <Input
        name="input11"
        placeholder="Input example (light) with clear"
        value={value}
        light
        onChange={setValue}
        onClear={() => setValue('')}
      />
    )
  },

  'Input': () => {
    const [value, setValue] = React.useState('')

    return (
      <Input
        name="input2"
        placeholder="Input example"
        value={value}
        onChange={setValue}
      />
    )
  },

  'Input with clear': () => {
    const [value, setValue] = React.useState('')

    return (
      <Input
        name="input21"
        placeholder="Input example with clear"
        value={value}
        onChange={setValue}
        onClear={() => setValue('')}
      />
    )
  },
}
