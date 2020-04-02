import * as React from 'react'
import { Input } from './Input'
import { Example } from './Example'

function LightInputExample() {
  const [value, setValue] = React.useState('')

  return (
    <Example title="Light input">
      <Input
        name="input1"
        placeholder="Input example (light)"
        value={value}
        light
        onChange={setValue}
      />
    </Example>
  )
}

function LightInputWithClearExample() {
  const [value, setValue] = React.useState('')

  return (
    <Example title="Light input with clear">
      <Input
        name="input11"
        placeholder="Input example (light) with clear"
        value={value}
        light
        onChange={setValue}
        onClear={() => setValue('')}
      />
    </Example>
  )
}

function InputExample() {
  const [value, setValue] = React.useState('')

  return (
    <Example title="Input">
      <Input
        name="input2"
        placeholder="Input example"
        value={value}
        onChange={setValue}
      />
    </Example>
  )
}

function InputWithClearExample() {
  const [value, setValue] = React.useState('')

  return (
    <Example title="Input with clear">
      <Input
        name="input21"
        placeholder="Input example with clear"
        value={value}
        onChange={setValue}
        onClear={() => setValue('')}
      />
    </Example>
  )
}

export function InputExamples() {
  return (
    <Example section title="Input">
      <LightInputExample />

      <LightInputWithClearExample />

      <InputExample />

      <InputWithClearExample />
    </Example>
  )
}
