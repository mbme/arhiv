import * as React from 'react'
import { Input } from './Input'
import { Example } from '../Example'
import { useForm } from './Form'

export function InputExamples() {
  const { Form } = useForm()

  return (
    <Example section title="Input">
      <Form>
        <Example title="Input">
          <Input
            name="input2"
            placeholder="Input example"
          />
        </Example>

        <Example title="Input with clear">
          <Input
            name="input21"
            placeholder="Input example with clear"
            withClear
          />
        </Example>
      </Form>
    </Example>
  )
}
