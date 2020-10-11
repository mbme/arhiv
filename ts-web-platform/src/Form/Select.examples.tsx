import * as React from 'react'
import { Select } from './Select'
import { Example } from '../Example'
import { useForm } from './Form'

export function SelectExamples() {
  const { Form } = useForm()

  return (
    <Example section title="Select">
      <Form>
        <Select
          label="Select"
          name="select"
          options={{ val1: 'val1', val2: 'val2' }}
        />
      </Form>
    </Example>
  )
}
