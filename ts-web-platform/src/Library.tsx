import * as React from 'react'

import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Select.examples'
import { InputExamples } from './Input.examples'
import { TextareaExamples } from './Textarea.examples'
import { IconExamples } from './Icon.examples'
import { StyleExamples } from './style/examples'
import { ConfirmationDialogExamples } from './Overlay/ConfirmationDialog.examples'

export function Library() {
  return (
    <div>
      <StyleExamples />

      <IconExamples />

      <ButtonExamples />

      <SelectExamples />

      <InputExamples />

      <TextareaExamples />

      <ConfirmationDialogExamples />
    </div>
  )
}
