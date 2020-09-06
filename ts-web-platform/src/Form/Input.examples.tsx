import * as React from 'react'
import { noop } from '@v/utils'
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
            onClear={noop}
          />
        </Example>
      </Form>
    </Example>
  )
}
