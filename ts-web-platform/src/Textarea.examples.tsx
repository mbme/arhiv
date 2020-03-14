import * as React from 'react'
import { Textarea } from './Textarea'

export const examples = {
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
