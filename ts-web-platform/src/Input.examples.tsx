import * as React from 'react'
import { Input } from './Input'
import { Examples } from './Examples'

const examples = {
  'Light input': function LightInputExample() {
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

  'Light input with clear': function LightInputWithClearExample() {
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

  'Input': function InputExample() {
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

  'Input with clear': function InputWithClearExample() {
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

export function InputExamples() {
  return (
    <Examples title="Input" examples={examples} />
  )
}
