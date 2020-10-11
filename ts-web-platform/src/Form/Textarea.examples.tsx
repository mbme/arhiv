import * as React from 'react'
import { Textarea } from './Textarea'
import { Example } from '../Example'
import { useForm } from './Form'

export function TextareaExamples() {
  const { Form } = useForm()

  return (
    <Example section title="Textarea">
      <Form>
        <Textarea
          label="Textarea"
          name="textarea"
          placeholder="Textarea example"
        />
      </Form>
    </Example>
  )
}
