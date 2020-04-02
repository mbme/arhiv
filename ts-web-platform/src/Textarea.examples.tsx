import * as React from 'react'
import { Textarea } from './Textarea'
import { Example } from './Example'

export function TextareaExamples() {
  const [value, setValue] = React.useState('')

  return (
    <Example section title="Textarea">
      <Textarea
        name="textarea"
        placeholder="Textarea example"
        value={value}
        onChange={setValue}
      />
    </Example>
  )
}
