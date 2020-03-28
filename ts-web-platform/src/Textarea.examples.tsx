import * as React from 'react'
import { Textarea } from './Textarea'
import { Examples } from './Examples'

const examples = {
  '': function TextareaExample() {
    const [value, setValue] = React.useState('')

    return (
      <Textarea
        name="textarea"
        placeholder="Textarea example"
        value={value}
        onChange={setValue}
      />
    )
  },
}

export function TextareaExamples() {
  return (
    <Examples title="Textarea" examples={examples} />
  )
}
